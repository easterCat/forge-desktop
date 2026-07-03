$vsPath = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"
$cargoBin = "$env:USERPROFILE\.cargo\bin"
$nodePath = "$env:USERPROFILE\nodejs"

# Use cmd to run vcvarsall and capture the env changes
$envBlock = cmd /c "`"$vsPath`" x64 >nul 2>&1 && set"
foreach ($line in $envBlock) {
    if ($line -match '^([^=]+)=(.*)$') {
        [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
    }
}

# Add cargo and node to PATH
$env:PATH = "$cargoBin;$nodePath;$env:PATH"

Write-Host "VS Environment configured successfully"
Write-Host "INCLUDE exists: $($env:INCLUDE.Length -gt 0)"
Write-Host "LIB exists: $($env:LIB.Length -gt 0)"

# Navigate to project directory
Set-Location "c:\Users\fuhuo\Desktop\OH-WorkSpace\forge-desktop"

# Run dev server
npm run dev
