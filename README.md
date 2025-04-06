# pgwire-lite-rs

## Testing

### vscode debug

- Config in `launch.json` is dependent on [the `CodeLLDB` extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb).
- Also installed [the `rust-analyzer` extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

### Manual Dream Run


Output of running, once github reaches rate limit:

```log
Query did some non-notify thing.
--- Notice 1 ---
sqlstate: 01000
severity: NOTICE
message: a notice level event has occurred
detail: http response status code: 403, response body: {"message":"API rate limit exceeded for 1.136.105.61. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID 5747:146C5C:C94FAE:1027DAA:67EDE45F and timestamp 2025-04-03 01:29:04 UTC.","documentation_url":"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api","status":"403"}
http response status code: 403, response body: {"message":"API rate limit exceeded for 1.136.105.61. (But here's the good news: Authenticated requests get a higher rate limit. Check out the documentation for more details.) If you reach out to GitHub Support for help, please include the request ID 5750:3CE2DD:C54830:FE766C:67EDE461 and timestamp 2025-04-03 01:29:05 UTC.","documentation_url":"https://docs.github.com/rest/overview/rate-limits-for-the-rest-api","status":"403"}

```