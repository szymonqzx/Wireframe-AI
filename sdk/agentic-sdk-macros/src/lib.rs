//! # agentic-sdk-macros — `#[module]` attribute macro
//!
//! Transforms an `impl Module for MyStruct` block into a fully wired module.
//!
//! ## Usage
//!
//! ```ignore
//! use agentic_sdk::{Module, Envelope};
//! use agentic_sdk_macros::module;
//!
//! struct MyModule;
//!
//! #[module(
//!     subscribes = ["task.submitted"],
//!     publishes  = ["task.enriched"],
//!     queue_group = "task_handler"
//! )]
//! impl Module for MyModule {
//!     async fn handle(&mut self, env: Envelope<serde_json::Value>) -> Vec<Envelope<serde_json::Value>> {
//!         vec![]
//!     }
//! }
//! ```
//!
//! Generated automatically:
//! - `Module::subscribes()` → returns the `subscribes` list
//! - `Module::publishes()` → returns the `publishes` list
//! - `MyModule::run(nats_url)` → connects to NATS, announces online,
//!   subscribes, dispatches handle() to incoming messages

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    token::Comma,
    *,
};

// ── Attribute argument parsing ──────────────────────────────────────────────

struct ModuleArgs {
    subscribes: Vec<String>,
    publishes: Vec<String>,
    queue_group: String,
}

impl Parse for ModuleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut subscribes = Vec::new();
        let mut publishes = Vec::new();
        let mut queue_group = String::from("default");

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            if ident == "subscribes" || ident == "publishes" {
                let content;
                bracketed!(content in input);
                let mut vals = Vec::new();
                while !content.is_empty() {
                    let s: LitStr = content.parse()?;
                    vals.push(s.value());
                    if !content.is_empty() {
                        let _: Comma = content.parse()?;
                    }
                }
                if ident == "subscribes" {
                    subscribes = vals;
                } else {
                    publishes = vals;
                }
            } else if ident == "queue_group" {
                let val: LitStr = input.parse()?;
                queue_group = val.value();
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    format!("unknown attribute: {}", ident),
                ));
            }

            if !input.is_empty() {
                let _: Option<Token![,]> = input.parse()?;
            }
        }

        Ok(ModuleArgs {
            subscribes,
            publishes,
            queue_group,
        })
    }
}

// ── Main proc-macro ────────────────────────────────────────────────────────

/// Attribute macro to transform an `impl Module for T` block into a
/// fully wired Wireframe AI module.
#[proc_macro_attribute]
pub fn module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ModuleArgs);
    let impl_block = parse_macro_input!(item as ItemImpl);

    // Extract the struct type name
    let struct_type = match &impl_block.self_ty.as_ref() {
        Type::Path(tp) => tp.path.segments.last().unwrap().ident.clone(),
        _ => {
            return syn::Error::new_spanned(
                &impl_block.self_ty,
                "#[module] only supports `impl Module for MyStruct`",
            )
            .to_compile_error()
            .into();
        }
    };

    // Extract handle method body from the impl block
    let handle_body = impl_block.items.iter().find_map(|item| {
        if let ImplItem::Fn(m) = item {
            if m.sig.ident == "handle" {
                return Some(&m.block);
            }
        }
        None
    });

    if handle_body.is_none() {
        return syn::Error::new_spanned(&impl_block, "#[module] requires a `handle` method")
            .to_compile_error()
            .into();
    }

    // Build string literals for subscribes/publishes arrays
    let sub_lits: Vec<LitStr> = args
        .subscribes
        .iter()
        .map(|s| LitStr::new(s, proc_macro2::Span::call_site()))
        .collect();
    let pub_lits: Vec<LitStr> = args
        .publishes
        .iter()
        .map(|s| LitStr::new(s, proc_macro2::Span::call_site()))
        .collect();

    let module_name = struct_type.to_string();
    let qg = LitStr::new(&args.queue_group, proc_macro2::Span::call_site());

    // Generate the expanded code
    let expanded = quote! {
        #[agentic_sdk::async_trait]
        impl agentic_sdk::Module for #struct_type {
            fn subscribes() -> &'static [&'static str] {
                &[#(#sub_lits),*]
            }

            fn publishes() -> &'static [&'static str] {
                &[#(#pub_lits),*]
            }

            async fn handle(&mut self, env: agentic_sdk::Envelope<serde_json::Value>)
                -> Vec<agentic_sdk::Envelope<serde_json::Value>>
            #handle_body
        }

        impl #struct_type {
            /// Run this module: connect to NATS, announce online, subscribe,
            /// and dispatch messages to handle().
            pub async fn run(mut self, nats_url: &str) -> Result<(), Box<dyn std::error::Error>>
            where
                Self: Sized + Send + 'static,
            {
                use agentic_sdk::{announce_online, announce_offline, start_heartbeat};
                use futures::StreamExt;

                let client = async_nats::connect(nats_url).await?;
                tracing::info!(url = %nats_url, "connected to NATS");

                announce_online(
                    &client,
                    #module_name,
                    "0.1.0",
                    <Self as agentic_sdk::Module>::subscribes(),
                    <Self as agentic_sdk::Module>::publishes(),
                )
                .await?;

                // Start heartbeat
                let _heartbeat = start_heartbeat(&client, #module_name, 30);

                // Graceful shutdown
                let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
                let shutdown_client = client.clone();
                tokio::spawn(async move {
                    tokio::signal::ctrl_c().await.expect("failed to listen for ctrl+c");
                    tracing::info!("received SIGINT — shutting down");
                    let _ = announce_offline(
                        &shutdown_client,
                        #module_name,
                        "0.1.0",
                    ).await;
                    let _ = shutdown_tx.send(true);
                });

                // Wrap self in Arc<Mutex> for concurrent access from subscriber tasks
                let module = std::sync::Arc::new(tokio::sync::Mutex::new(self));

                // Subscribe to each topic with queue group
                let mut handles = Vec::new();
                for topic in <Self as agentic_sdk::Module>::subscribes() {
                    let topic_str: String = topic.to_string();
                    let qg_str: String = #qg.to_string();
                    let sub = client
                        .queue_subscribe(topic_str, qg_str)
                        .await?;
                    let nc = client.clone();
                    let module = module.clone();
                    let mut shutdown_rx = shutdown_rx.clone();
                    let handle = tokio::spawn(async move {
                        let mut sub = sub;
                        loop {
                            tokio::select! {
                                msg = sub.next() => {
                                    match msg {
                                        Some(msg) => {
                                            let env: agentic_sdk::Envelope<serde_json::Value> =
                                                match serde_json::from_slice(&msg.payload) {
                                                    Ok(e) => e,
                                                    Err(e) => {
                                                        tracing::error!(error = ?e, "failed to parse envelope");
                                                        continue;
                                                    }
                                                };
                                            let results = module.lock().await.handle(env).await;
                                            for result_env in results {
                                                if let Ok(data) = serde_json::to_vec(&result_env) {
                                                    let topic = result_env.topic.clone();
                                                    let _ = nc.publish(topic, data.into()).await;
                                                }
                                            }
                                        }
                                        None => break,
                                    }
                                }
                                _ = shutdown_rx.changed() => {
                                    if *shutdown_rx.borrow() {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                    handles.push(handle);
                }

                tracing::info!("{} ready — subscribed to {:?}", #module_name, <Self as agentic_sdk::Module>::subscribes());

                // Wait for shutdown signal
                let mut shutdown_rx_main = shutdown_rx.clone();
                shutdown_rx_main.changed().await.ok();

                // Wait for all subscriber tasks to finish
                for handle in handles {
                    let _ = handle.await;
                }

                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
