# Starts the title with the settings this fork is measured with on an RX 9070 XT 16 GB and 32 GB of RAM
# (README.md, "Where it stands"): play.ps1's settings and the ones below.
#   scripts\play-9070xt.ps1 -Exe <path of your own copy's executable>
# -Warm       does not start the title: makes the host pipelines earlier runs drew with, under these settings.
#             Do it once after a first run, and again after a new build or driver: the run after it does not
#             stutter while the driver makes pipelines.
# -NoPorts    without the renderer patches (the first thing to try when draws go missing or the device is removed)
# -Safe       play.ps1's -Safe: without the list calls translated by threads of ours. About a third slower, and
#             without the meshes drawn as sheets of polygons that come with them (README, "What is known to be wrong")
# -Output     the size the title renders its output at (2560x1440; 1920x1080 takes less of the card)
# -More       other settings, as play.ps1: -More @{VICTORY_MOUSE='0'}
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Warm,
    [switch]$NoPorts,
    [switch]$Safe,
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
    # The wide panel in colours (F3 shows and hides it).
    VICTORY_PANEL = '2'
    # play.ps1 reads title memory in place wherever it can since kripotker's memory series. On this PC (a card with
    # memory of its own) that opened and forgot thousands of small regions while the title streamed, and the frame
    # rate swung between 2 and 20 fps: left as before here.
    VICTORY_PORT_ALL = '0'; VICTORY_INPLACE = '0'; VICTORY_WINDOW_KB = '4096'
}
if (-not $NoPorts) {
    # (Not renderer patch 1, VICTORY_PORT_CONSTANTS: with it the sky flickered between dark and light here.)
    foreach ($port in 'DESCRIPTORS', 'CBV', 'GUARD', 'JOURNAL') { $settings["VICTORY_PORT_$port"] = '1' }
}
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }
& (Join-Path $PSScriptRoot 'play.ps1') -Exe $Exe -More $settings -Warm:$Warm -Safe:$Safe
