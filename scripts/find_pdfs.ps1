$vfsPath = 'C:\Users\1\AppData\Roaming\com.deepstudent.app\slots\slotB\vfs_blobs'
$count = 0
$pdfFiles = @()
Get-ChildItem $vfsPath -Recurse -File | ForEach-Object {
    $f = $_
    $bytes = [System.IO.File]::ReadAllBytes($f.FullName)
    if ($bytes.Length -gt 4 -and $bytes[0] -eq 0x25 -and $bytes[1] -eq 0x50 -and $bytes[2] -eq 0x44 -and $bytes[3] -eq 0x46) {
        $count++
        if ($pdfFiles.Count -lt 10) {
            $sizeKB = [math]::Round($bytes.Length/1024, 0)
            $pdfFiles += "$($f.Name) ($sizeKB KB)"
        }
    }
}
Write-Output "=== PDF files found in VFS ==="
$pdfFiles | ForEach-Object { Write-Output $_ }
Write-Output "Total PDF blobs: $count"
Write-Output "Total VFS blobs scanned: $((Get-ChildItem $vfsPath -Recurse -File).Count)"
