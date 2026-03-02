# last-commit-rs

lightweight and overengineered rust http server to replace [last-commit-worker](https://github.com/feederbox826/last-commit-worker) and move off of cloudflare

Used to be a serverless, stateless worker that had a p95 of >100ms to fetch from KV, now it's all from memory and uses <1M to store all existing entries. Serializes into JSON and has a less robust TTL but boy is it fast and efficient