param(
  [Parameter(Mandatory = $true)][string]$Command
)

$ErrorActionPreference = 'Stop'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$nsisBin = 'C:\Program Files (x86)\NSIS'
$env:PATH = "$cargoBin;$nsisBin;$env:PATH"
$vcvars = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat'
if (-not (Test-Path -LiteralPath $vcvars)) {
  throw "vcvars64.bat tidak ditemukan: $vcvars"
}
$full = "`"$vcvars`" && $Command"
cmd /c $full
if ($LASTEXITCODE -ne 0) {
  throw "Perintah gagal dengan exit code $LASTEXITCODE"
}
