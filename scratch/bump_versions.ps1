$files = Get-ChildItem -Recurse Cargo.toml
foreach ($file in $files) {
    if ($file.FullName -notmatch "target") {
        (Get-Content $file.FullName) -replace 'version = "0.1.0"', 'version = "1.0.0"' | Set-Content $file.FullName
    }
}
(Get-Content frontend/package.json) -replace '"version": "0.1.0"', '"version": "1.0.0"' | Set-Content frontend/package.json
