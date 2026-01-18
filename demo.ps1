Write-Host "🧪 Running Alchemist Demo..." -ForegroundColor Cyan

# 1. JSON to TypeScript
Write-Host "`n1. Converting Complex JSON to TypeScript..." -ForegroundColor Yellow
cargo run -q -- -i examples/complex_api_response.json -t typescript

# 2. TOML to Python
Write-Host "`n2. Converting TOML Config to Python (Pydantic)..." -ForegroundColor Yellow
cargo run -q -- -i examples/app_config.toml -f toml -t python -n AppConfig

# 3. YAML to Rust
Write-Host "`n3. Converting Kubernetes YAML to Rust..." -ForegroundColor Yellow
cargo run -q -- -i examples/k8s_deployment.yaml -f yaml -t rust -n K8sDeployment

Write-Host "`n4. Testing Map Detection & Stdin (JSON -> Rust)..." -ForegroundColor Cyan
Get-Content examples\map_test.json | cargo run -- -q -t rust --root-name SystemConfig

Write-Host "`n✨ Demo Complete!" -ForegroundColor Green
