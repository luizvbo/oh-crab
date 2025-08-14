# This creates a 10GB dev drive, and exports all required environment
# variables so that rustup, cargo, and other tools all use the dev drive as much
# as possible for better performance.

$Drive = "D:"
$Tmp = "$($Drive)\ohcrab-tmp"

# Create the directory ahead of time in an attempt to avoid race-conditions
New-Item $Tmp -ItemType Directory

# Set environment variables for the rest of the job
Write-Output `
	"DEV_DRIVE=$($Drive)" `
	"TMP=$($Tmp)" `
	"TEMP=$($Tmp)" `
	"RUSTUP_HOME=$($Drive)/.rustup" `
	"CARGO_HOME=$($Drive)/.cargo" `
	>> $env:GITHUB_ENV
