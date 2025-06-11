$lpcs = @('lpc81x', 'lpc11xx', 'lpc13xx', 'lpc15xx', 'lpc17xx')

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