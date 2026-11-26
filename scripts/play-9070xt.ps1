# Starts the title with the settings this fork is measured with on an RX 9070 XT 16 GB and 32 GB of RAM
# (README.md, "Where it stands"): play.ps1's settings and the ones below.
#   scripts\play-9070xt.ps1 -Exe <path of your own copy's executable>
# -Warm       does not start the title: makes the host pipelines earlier runs drew with, under these settings.
#             Do it once after a first run, and again after a new build or driver: the run after it does not
#             stutter while the driver makes pipelines.
# -NoPorts    without the renderer patches (the first thing to try when draws go missing or the device is removed)
# -Output     the size the title renders its output at (2560x1440; 1920x1080 takes less of the card)
# -More       other settings, as play.ps1: -More @{VICTORY_MOUSE='0'}
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Warm,
    [switch]$NoPorts,
    [string]$Output = '2560x1440',
    [hashtable]$More = @{}
)
$settings = @{
    # The title's own output size instead of 4K, and host copies of streamed textures without their largest
    # levels as the card fills: a 16 GB card stays inside its budget.
    VICTORY_OUTPUT = $Output; VICTORY_TEXTURE_MAX = 'auto'
    # Texture slots that wait for their memory are looked at every half second (it was 8 to 20 s at 12 fps),
    # and a texture whose memory is away keeps what it showed for 15 s (it was 3).
    VICTORY_TEXTURE_LOOK_MS = '500'; VICTORY_TEXTURE_AWAY_MS = '15000'
    # The light in shadow, the mouse as the right stick with a PC key layout, and the list -Warm reads.
    VICTORY_AMBIENT_AIR = '1.5'; VICTORY_MOUSE = '1'; VICTORY_PIPELINE_BOOK = '1'
}
if (-not $NoPorts) {
    foreach ($port in 'CONSTANTS', 'DESCRIPTORS', 'CBV', 'GUARD', 'JOURNAL') { $settings["VICTORY_PORT_$port"] = '1' }
}
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }
& (Join-Path $PSScriptRoot 'play.ps1') -Exe $Exe -More $settings -Warm:$Warm
