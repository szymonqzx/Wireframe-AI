---
name: deployment
description: Production deployment with pre-flight checks, deployment execution, and verification. Use for deploying applications to production or staging environments.
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - model
---

# Production Deployment

"Production deployment with pre-flight checks, deployment execution, and verification."

## When to Use
- Deploying applications to production or staging environments
- Running pre-deployment checks and validation
- Rolling back to previous versions
- Deploying to preview environments for testing

## When NOT to Use
- Local development or testing (use preview instead)
- Non-deployment tasks (use enhancement skill for code changes)
- Emergency hotfixes without proper testing
- Deploying uncommitted or untested code

## Pre-Deployment Checklist

Before any deployment:

```markdown
## 🚀 Pre-Deploy Checklist

### Code Quality
- [ ] No compilation/type errors (adapt to your project: `npx tsc --noEmit`, `cargo check`, `python -m py_compile`, etc.)
- [ ] Linting passing (adapt to your project: `npx eslint .`, `cargo clippy`, `flake8`, etc.)
- [ ] All tests passing (adapt to your project: `npm test`, `cargo test`, `pytest`, etc.)

### Security
- [ ] No hardcoded secrets
- [ ] Environment variables documented
- [ ] Dependencies audited (adapt to your project: `npm audit`, `cargo audit`, `pip-audit`, etc.)

### Performance
- [ ] Bundle size acceptable
- [ ] No console.log statements
- [ ] Images optimized

### Documentation
- [ ] README updated
- [ ] CHANGELOG updated
- [ ] API docs current

### Ready to deploy? (y/n)
```

## Deployment Flow

```
┌─────────────────┐
│  deployment     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Pre-flight     │
│  checks         │
└────────┬────────┘
         │
    Pass? ──No──► Fix issues
         │
        Yes
         │
         ▼
┌─────────────────┐
│  Build          │
│  application    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Deploy to      │
│  platform       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Health check   │
│  & verify       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  ✅ Complete    │
└─────────────────┘
```

## Platform Support

| Platform | Command | Notes |
|----------|---------|-------|
| Vercel | `vercel --prod` | For Next.js/React apps |
| Railway | `railway up` | Needs Railway CLI |
| Fly.io | `fly deploy` | Needs flyctl |
| Docker | `docker compose up -d` | For containerized apps |
| AWS | `eb deploy` | Elastic Beanstalk |
| Heroku | `git push heroku main` | For Heroku apps |
| Custom | [your deploy command] | Adapt to your platform |

## Output Format

### Successful Deploy

```markdown
## 🚀 Deployment Complete

### Summary
- **Version:** v1.2.3
- **Environment:** production
- **Duration:** 47 seconds
- **Platform:** Vercel

### URLs
- 🌐 Production: https://app.example.com
- 📊 Dashboard: https://vercel.com/project

### What Changed
- Added user profile feature
- Fixed login bug
- Updated dependencies

### Health Check
✅ API responding (200 OK)
✅ Database connected
✅ All services healthy
```

### Failed Deploy

```markdown
## ❌ Deployment Failed

### Error
Build failed at step: [specific build step]

### Details
```
[error output]
```

### Resolution
1. Fix error in [file:line]
2. Run <build-command> locally to verify
3. Try deployment again

### Rollback Available
Previous version (v1.2.2) is still active.
Run rollback if needed.
```

## Edge Case Handling
- **Deployment conflicts:** Multiple deployments in progress - queue deployments or notify user
- **Environment mismatch:** Staging config differs from production - validate config before deploy
- **Rollback failure:** Rollback doesn't work - test rollback procedure in staging first
- **Partial deployment:** Some services deploy but others fail - use canary deployments
- **Database migration failures:** Migration fails mid-deploy - use reversible migrations

## Failure Modes
- **Pre-flight check failures:** Tests or lint fails - fix issues before proceeding
- **Build failures:** Compilation or bundling fails - check logs for specific errors
- **Deployment timeout:** Deployment takes too long - set timeout and auto-rollback
- **Health check failures:** New deployment unhealthy - auto-rollback to previous version
- **Configuration errors:** Wrong environment variables - validate config schema before deploy

## Performance Considerations
- **Deployment time:** Optimize for fast deployments (parallel builds, caching)
- **Zero-downtime:** Use blue-green or canary deployments for production
- **Rollback speed:** Ensure rollback completes within 30 seconds
- **Build artifacts:** Cache dependencies to speed up builds
- **CDN propagation:** Account for CDN cache invalidation time

## Security Notes
- **Secrets validation:** Verify no secrets in code before deployment
- **Environment isolation:** Ensure production secrets are separate from staging
- **Access control:** Restrict deployment permissions to authorized team members
- **Audit logging:** Log all deployments with who, when, and what changed
- **Vulnerability scanning:** Run security scans before production deployment

## Guardrails
- Always run pre-flight checks before deployment
- Ensure all tests pass before production deployment
- Verify environment variables and secrets are configured
- Have rollback plan ready before production deploy
- Test in preview/staging before production

## Related Skills

- `powershell-windows` - PowerShell Windows patterns for deployment scripts
