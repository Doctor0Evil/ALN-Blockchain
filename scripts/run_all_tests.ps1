<# PowerShell script to run Node + Rust tests on Windows #>
Set-StrictMode -Version Latest
Write-Host "Running Node.js tests..."
npm ci
npm run test --workspace=aln/tests
npm run test --workspace=aln/core

Write-Host "Running all Rust workspace tests..."
cargo test --all --workspace --verbose

Write-Host "Running Rust cw-multi-test integration tests..."
Push-Location tests/integration
cargo test --manifest-path Cargo.toml --verbose
Pop-Location

Write-Host "All tests completed."
