$FLAG = "D3CTF(TECH-EV0LVE,EMBR@C3-PR0GR3SS)"
$PROMPT_SUCCESS = "NOW MARCH BEYOND, AND REVIVE THE LEGACY."
$PROMPT_FAILURE = "THAT IS NOT CORRECT. TRY AGAIN :D"

if (-not (Test-Path -Path "Cargo.toml")) {
    Write-Error "E: The script must be run in the project root folder"
    exit 1
}

cargo build --profile "ctf_ftr" --quiet

if (-not (Test-Path -Path "target\ctf_ftr\modern_legacy.exe")) {
    Write-Error "E: The build failed"
    exit 2
}

New-Item -ItemType Directory -Path "target\artifacts" -Force | Out-Null

if (-not "$($FLAG | & 'target\ctf_ftr\modern_legacy.exe')".Contains($PROMPT_SUCCESS)) {
    Write-Error "E: Flag `"$FLAG`" is incorrect; check your code again"
    exit 3
}
if (-not "$(-join (Get-Random -Count $FLAG.Length -Minimum 65 -Maximum 91 | ForEach-Object { [char]$_ }) | & 'target\ctf_ftr\modern_legacy.exe')".Contains($PROMPT_FAILURE)) {
    Write-Error "E: False positive detected; check your code again"
    exit 3
}

Compress-Archive -Path "assets\*" -DestinationPath "target\artifacts\writeup.zip" -Force
Compress-Archive -Path "target\ctf_ftr\modern_legacy.exe" -DestinationPath "target\artifacts\modern_legacy.zip" -Force
