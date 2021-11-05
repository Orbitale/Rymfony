
# The goal here is to
# Check releases, and update the associated data here.
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

Write-Output "Caddy was successfully downloaded to $output_path\$FileName"
