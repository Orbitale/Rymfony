
# The goal here is to
# Check releases and their checksums, and update the associated data here.
# @link https://github.com/caddyserver/caddy/releases
$caddy_version = "2.4.3"

# We only support this version for now.
# Windows won't probably be the most used platform anyway.
$release_filename = "windows_amd64"

# Download Caddy
$binary_url = "https://github.com/caddyserver/caddy/releases/download/v${caddy_version}/caddy_${caddy_version}_${release_filename}.zip"
$tmpfile_caddy = New-TemporaryFile
Invoke-WebRequest -uri $binary_url -OutFile $tmpfile_caddy

# Output it to bin/caddy.exe
$output_path = Write-Output $pwd\bin
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($tmpfile_caddy) # Open ZIP archive for reading
# Find all files in ZIP that match the filter "caddy.exe"
$zip.Entries |
        Where-Object { $_.FullName -like 'caddy.exe' } |
        ForEach-Object {
            # extract the selected items from the ZIP archive and copy them to the output folder
            $FileName = $_.Name
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($_, "$output_path\$FileName", $true)
        }
$zip.Dispose() # Close ZIP file

# Remove temporary file
rm $tmpfile_caddy

# Check hash
$caddy_hash = (Get-FileHash -Algorithm SHA512 .\bin\caddy.exe).Hash

# Download list of checksums for further validation
$checksums_url = "https://github.com/caddyserver/caddy/releases/download/v${caddy_version}/caddy_${caddy_version}_checksums.txt"
$checksums = (Invoke-WebRequest -uri $checksums_url).toString()

if ($checksums -match "${caddy_version}_windows_amd64\.zip") {
    Write-Output "Caddy was successfully downloaded to $output_path\$FileName"
} else {
    Write-Output "Invalid checksum for the downloaded file."

    exit 1
}
