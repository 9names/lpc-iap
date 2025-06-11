$lpcs = @('lpc81x', 'lpc11xx', 'lpc13xx', 'lpc15xx', 'lpc177x_8x')

Push-Location "lpc-probers-target"
foreach ($lpc in $lpcs) {
    $jsonPath = "..\docs\$($lpc)_targets.json"
    $generated = "$($lpc)_generated.yaml"
    $template = "..\algos\$($lpc)\template.yaml"

    # Write-Host "json: $jsonPath, generated: $generated, template: $template"

    cargo run -- $jsonPath
    Copy-Item "$generated" "$template"
}
Pop-Location

foreach ($lpc in $lpcs) {
    Push-Location "algos\$lpc"
    cargo build --release

    $v6mPath = "target\thumbv6m-none-eabi\release\$lpc"
    $v7mPath = "target\thumbv7m-none-eabi\release\$lpc"

    if (Test-Path $v6mPath) {
        target-gen elf $v6mPath -u template.yaml
        Copy-Item template.yaml "..\..\$lpc.yaml" -Force
    }
    elseif (Test-Path $v7mPath) {
        target-gen elf $v7mPath -u template.yaml
        Copy-Item template.yaml "..\..\$lpc.yaml" -Force
    }
    else {
        Write-Host "No output found for target $lpc"
    }
    Pop-Location
}
