---
name: github-mcp
description: GitHub MCP server usage patterns and best practices for GitHub API operations
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# GitHub MCP

## Purpose

Guide effective use of GitHub MCP servers for repository operations, issue management, pull requests, and code search. Ensure correct tool selection between GitHub MCP and GitKraken MCP based on provider support and task requirements.

## When to Use

Use this skill when:
- Interacting with GitHub repositories (issues, pull requests, code search)
- Creating or managing GitHub issues and pull requests
- Searching repositories, code, or issues across GitHub
- Retrieving repository metadata and file contents
- Managing pull request reviews and comments
- Working with multiple git providers (GitLab, Azure, Jira, Linear)

## Protocol

### Step 1: Choose the Right MCP Server

1. **Identify Your Provider**
   - GitHub only: Use GitHub MCP
   - Multiple providers (GitLab, Azure, Jira, Linear): Use GitKraken MCP
   - Uncertain: Check tool selection guide below

2. **Check Tool Availability**
   - Review the tool selection guide for your specific task
   - Verify the MCP server supports your required operations
   - Consider future multi-provider needs

### Step 2: Repository Operations

1. **Get Repository Information**
   - Basic repo info: `mcp3_github_get_repository`
   - File content: `mcp0_repository_get_file_content` (GitKraken MCP)
   - Always include provider, repository_name, repository_organization for GitKraken

2. **Handle Authentication**
   - Ensure proper authentication tokens are configured
   - Check for API rate limits
   - Handle authentication errors gracefully

### Step 3: Issue Management

1. **List Issues**
   - GitHub-specific: `mcp3_github_list_issues`
   - Multi-provider: `mcp0_issues_assigned_to_me`
   - Use state parameter (open/closed/all)
   - Handle pagination with limit/offset

2. **Create and Manage Issues**
   - Create issue: `mcp3_github_create_issue`
   - Get details: `mcp0_issues_get_detail`
   - Add comments: `mcp0_issues_add_comment`
   - Include proper provider parameters

### Step 4: Pull Request Operations

1. **List and Get PRs**
   - List PRs: `mcp3_github_list_pull_requests`
   - Get details: `mcp3_github_get_pull_request` or `mcp0_pull_request_get_detail`
   - Use state parameter for filtering
   - Handle large result sets with pagination

2. **Create and Review PRs**
   - Create PR: `mcp0_pull_request_create`
   - Create review: `mcp0_pull_request_create_review`
   - Get comments: `mcp0_pull_request_get_comments`
   - Specify draft status if needed

### Step 5: Search Operations

1. **Construct Search Queries**
   - Use proper GitHub search syntax: `q=language:rust+example`
   - Be specific to reduce result sets
   - Use filters: repo:, stars:, language:, is:

2. **Handle Search Results**
   - Code search: `mcp3_github_search_code`
   - Issues/PRs: `mcp3_github_search_issues_and_prs`
   - Repositories: `mcp3_github_search_repositories`
   - Process results with pagination

### Step 6: Error Handling and Best Practices

1. **Handle Common Errors**
   - API rate limits: Implement backoff
   - Authentication failures: Check token configuration
   - Not found errors: Verify repository exists
   - Permission errors: Check access rights

2. **Optimize Performance**
   - Cache repository metadata (rarely changes)
   - Use specific queries to reduce result sets
   - Handle pagination for large datasets
   - Batch operations when possible

## Key Patterns

### Repository Operations

#### Get Repository Details
```python
# Use mcp3_github_get_repository for basic repo info
# Parameters: owner, repo
```

#### Get File Content from Repository
```python
# Use mcp0_repository_get_file_content (GitKraken MCP)
# Parameters: provider, repository_name, repository_organization, ref, file_path
# Supports GitHub, GitLab, Bitbucket, Azure
```

### Issue Management

#### List Issues
```python
# Use mcp3_github_list_issues (GitHub MCP)
# Parameters: owner, repo, state (open/closed/all), limit, offset

# Use mcp0_issues_assigned_to_me (GitKraken MCP)
# Parameters: provider (github/gitlab/jira/azure/linear), page
# For fetching issues assigned to current user
```

#### Get Issue Details
```python
# Use mcp0_issues_get_detail (GitKraken MCP)
# Parameters: provider, issue_id, repository_name, repository_organization (for GitHub/GitLab)
# Supports GitHub/GitLab/Jira/Azure/Linear
```

#### Create Issue
```python
# Use mcp3_github_create_issue (GitHub MCP)
# Parameters: owner, repo, title, body
```

#### Add Comment to Issue
```python
# Use mcp0_issues_add_comment (GitKraken MCP)
# Parameters: provider, issue_id, comment, repository_name, repository_organization (for GitHub/GitLab)
```

### Pull Request Operations

#### List Pull Requests
```python
# Use mcp3_github_list_pull_requests (GitHub MCP)
# Parameters: owner, repo, state (open/closed/all), limit, offset

# Use mcp0_pull_request_assigned_to_me (GitKraken MCP)
# Parameters: provider, reviewer (include if reviewer), repository_name, repository_organization (for Azure/Bitbucket)
```

#### Get Pull Request Details
```python
# Use mcp3_github_get_pull_request (GitHub MCP)
# Parameters: owner, repo, pull_number

# Use mcp0_pull_request_get_detail (GitKraken MCP)
# Parameters: provider, pull_request_id, repository_name, repository_organization, pull_request_files (optional)
# Supports GitHub/GitLab/Bitbucket/Azure
```

#### Create Pull Request
```python
# Use mcp0_pull_request_create (GitKraken MCP)
# Parameters: provider, repository_name, repository_organization, title, source_branch, target_branch, body (optional), is_draft (optional)
# Supports GitHub/GitLab/Bitbucket/Azure
```

#### Create Pull Request Review
```python
# Use mcp0_pull_request_create_review (GitKraken MCP)
# Parameters: provider, pull_request_id, repository_name, repository_organization, review, approve (optional)
```

#### Get Pull Request Comments
```python
# Use mcp0_pull_request_get_comments (GitKraken MCP)
# Parameters: provider, pull_request_id, repository_name, repository_organization
```

### Search Operations

#### Search Code
```python
# Use mcp3_github_search_code (GitHub MCP)
# Parameters: query, limit, offset
# Query syntax: q=language:rust+example
```

#### Search Issues and Pull Requests
```python
# Use mcp3_github_search_issues_and_prs (GitHub MCP)
# Parameters: query, limit, offset
# Query syntax: q=repo:owner/name+is:issue+state:open
```

#### Search Repositories
```python
# Use mcp3_github_search_repositories (GitHub MCP)
# Parameters: query, limit, offset
# Query syntax: q=language:rust+stars:>100
```

## Common Pitfalls

- **Wrong MCP server**: Using GitHub MCP when GitKraken MCP has broader provider support (GitLab, Azure, Jira, Linear)
- **Missing parameters**: Forgetting repository_name and repository_organization for GitKraken MCP GitHub operations
- **Provider confusion**: Not specifying correct provider parameter in GitKraken MCP tools
- **Query syntax**: Not using proper GitHub search query syntax (q= parameter)
- **Pagination**: Not handling limit/offset parameters for large result sets

## Best Practices

- **Prefer GitKraken MCP** for multi-provider support (GitHub, GitLab, Bitbucket, Azure, Jira, Linear)
- **Use GitHub MCP** for GitHub-specific operations or when provider is always GitHub
- **Check provider support**: Verify the tool supports your git provider before using
- **Handle pagination**: Use limit and offset parameters for large datasets
- **Cache results**: Repository metadata rarely changes, consider caching
- **Error handling**: Always check for API rate limits and authentication errors
- **Use search wisely**: Construct specific queries to reduce result sets

## Tool Selection Guide

| Task | GitHub MCP | GitKraken MCP |
|------|------------|---------------|
| Basic repo info | ✅ get_repository | ❌ |
| File content | ❌ | ✅ repository_get_file_content |
| List issues | ✅ list_issues | ✅ issues_assigned_to_me |
| Issue details | ❌ | ✅ issues_get_detail |
| Create issue | ✅ create_issue | ❌ |
| Add comment | ❌ | ✅ issues_add_comment |
| List PRs | ✅ list_pull_requests | ✅ pull_request_assigned_to_me |
| PR details | ✅ get_pull_request | ✅ pull_request_get_detail |
| Create PR | ❌ | ✅ pull_request_create |
| PR review | ❌ | ✅ pull_request_create_review |
| PR comments | ❌ | ✅ pull_request_get_comments |
| Search code | ✅ search_code | ❌ |
| Search issues/PRs | ✅ search_issues_and_prs | ❌ |
| Search repos | ✅ search_repositories | ❌ |

## Integration

This skill integrates with:
- `/code-fix` - For fixing issues identified in code review
- `/implementation` - For implementing features based on repository analysis
- `/clean-code` - For code quality standards during review
- `/architecture` - For architectural decisions based on codebase analysis
