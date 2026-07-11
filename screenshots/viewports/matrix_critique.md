# Viewport matrix critiques
PRE-PROD: all BAD must be none before push.
Run: 2026-07-11 FIX-LOOP 2 (mtime>=run_start_unix; re-reviewed units supersede prior BADs)
NOTE: lines rewritten as each unit re-reviewed on NEW artifacts only.
---

## pipeline batch
CRITIQUE phone_android_01_boot: GOOD: title RUSTY DASHER + ENTER/click/tap CTA settled (not mid-WASM); portrait 720×1600 emu; no Translate; qa_matrix URL | BAD: none (lab Chrome URL bar OK for full-display emulator path)
CRITIQUE phone_android_02_menu: GOOD: stick/DASH phone copy + swap strip; Best 2; portrait; no Translate | BAD: none
CRITIQUE phone_android_03_mode_select: GOOD: SELECT MODE all 4 modes + NORMAL + green START + touch hints; portrait | BAD: none
CRITIQUE phone_android_04_playing: GOOD: mid-play field + stars/hazards; bottom stick+DASH chrome outside field; Score/hearts HUD; portrait | BAD: none
CRITIQUE phone_android_05_game_over: GOOD: real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 stats + touch hints; portrait 720×1600; force_go armed | BAD: none
CRITIQUE phone_android_landscape_01_boot: GOOD: title RUSTY DASHER + ENTER/click/tap CTA settled (not mid-WASM); landscape 1600×720; **Translate FIXED**; **mid-download FIXED**; qa_matrix URL intact | BAD: none (lab Chrome URL bar outside game canvas OK for full-display emulator path)
CRITIQUE phone_android_landscape_02_menu: GOOD: real menu panel — stick/DASH phone copy + swap strip; landscape 1600×720; **Translate FIXED** | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_03_mode_select: GOOD: SELECT MODE all 4 modes + NORMAL + green START + touch hints; landscape; **Translate FIXED** | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_04_playing: GOOD: mid-play field + stars; left stick + right DASH chrome outside field; Score/hearts HUD; landscape; **Translate FIXED** | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_05_game_over: GOOD: real **GAME OVER** unclipped — CLASSIC/NORMAL Score 0 stats + touch hints; landscape 1600×720; **force_go FIXED** (qa_matrix+qa_go_ms URL no longer shell-truncated) | BAD: none
CRITIQUE phone_portrait_01_boot: GOOD: title RUSTY DASHER + ENTER/click/tap CTA settled; portrait 738×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_portrait_02_menu: GOOD: stick/DASH phone copy + swap; panel centered; portrait | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: modes+difficulty+START + touch hints readable; portrait | BAD: none
CRITIQUE phone_portrait_04_playing: GOOD: mid-play field; bottom stick+DASH chrome outside field; HUD legible; portrait | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 + touch hints; portrait 738×1600; force_go | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: title RUSTY DASHER + CTA settled; landscape 1600×738 emu; full qa_matrix URL; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_landscape_02_menu: GOOD: stick/DASH copy + swap; panel fits landscape | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: modes+diff+wide START; hints readable landscape | BAD: none
CRITIQUE phone_landscape_04_playing: GOOD: PSP left stick + right DASH outside field; HUD score/hearts; mid-play entities; landscape | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: real **GAME OVER** unclipped — CLASSIC/NORMAL Score 0 + touch hints; landscape 1600×738; force_go | BAD: none

## qhd + 4k
CRITIQUE qhd_01_boot: GOOD: title+CTA centered readable at QHD | BAD: none
CRITIQUE qhd_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome | BAD: none
CRITIQUE qhd_03_mode_select: GOOD: modes+difficulty readable | BAD: none
CRITIQUE qhd_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible | BAD: none
CRITIQUE qhd_05_game_over: GOOD: real NEW HIGH SCORE! unclipped on desktop QHD; stats Score 2 + keyboard hints | BAD: none
CRITIQUE 4k_01_boot: GOOD: title+CTA centered readable at 4K residual NEW (visual A4b) | BAD: none
CRITIQUE 4k_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome residual NEW (visual A4b) | BAD: none
CRITIQUE 4k_03_mode_select: GOOD: modes+difficulty readable at 4K residual NEW (visual A4b) | BAD: none
CRITIQUE 4k_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible residual NEW (visual A4b) | BAD: none
CRITIQUE 4k_05_game_over: GOOD: real GAME OVER unclipped Score 0; desktop ENTER/SPACE/ESC hints residual NEW (visual A4b) | BAD: none

## desktop batch (laptop_hd laptop_scaled laptop_720 1080p)
CRITIQUE laptop_hd_01_boot: GOOD: title+CTA centered readable at 1366×768 residual NEW (visual A4b) | BAD: none
CRITIQUE laptop_hd_02_menu: GOOD: desktop WASD/mouse control copy; no touch chrome residual NEW (visual A4b) | BAD: none
CRITIQUE laptop_hd_03_mode_select: GOOD: modes+difficulty row readable; keyboard hints residual NEW (visual A4b) | BAD: none
CRITIQUE laptop_hd_04_playing: GOOD: full field; no stick/DASH chrome; Dash READY; HUD legible residual NEW (visual A4b) | BAD: none
CRITIQUE laptop_hd_05_game_over: GOOD: real NEW HIGH SCORE! unclipped Score 2; desktop ENTER/SPACE/ESC hints residual NEW (visual A4b) | BAD: none
CRITIQUE laptop_scaled_01_boot: GOOD: title+CTA centered readable | BAD: none
CRITIQUE laptop_scaled_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome | BAD: none
CRITIQUE laptop_scaled_03_mode_select: GOOD: modes+difficulty readable | BAD: none
CRITIQUE laptop_scaled_04_playing: GOOD: full field no stick chrome; Dash READY; HUD OK | BAD: none
CRITIQUE laptop_scaled_05_game_over: GOOD: NEW HIGH SCORE unclipped; desktop keyboard hints | BAD: none
CRITIQUE laptop_720_01_boot: GOOD: title+CTA centered readable | BAD: none
CRITIQUE laptop_720_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome | BAD: none
CRITIQUE laptop_720_03_mode_select: GOOD: modes+difficulty readable | BAD: none
CRITIQUE laptop_720_04_playing: GOOD: full field no stick chrome; Dash READY | BAD: none
CRITIQUE laptop_720_05_game_over: GOOD: real GAME OVER unclipped; desktop keyboard hints | BAD: none
CRITIQUE 1080p_01_boot: GOOD: title+CTA centered readable | BAD: none
CRITIQUE 1080p_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome | BAD: none
CRITIQUE 1080p_03_mode_select: GOOD: modes+difficulty readable | BAD: none
CRITIQUE 1080p_04_playing: GOOD: full field no stick chrome; Dash READY | BAD: none
CRITIQUE 1080p_05_game_over: GOOD: full GAME OVER unclipped; desktop ENTER/SPACE/ESC hints | BAD: none
CRITIQUE phone_large_01_boot: GOOD: title RUSTY DASHER + CTA settled (not mid-WASM); portrait 738×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_large_02_menu: GOOD: stick/DASH phone copy + swap strip; layout clean; portrait | BAD: none
CRITIQUE phone_large_03_mode_select: GOOD: modes+diff+START+hints readable; portrait | BAD: none
CRITIQUE phone_large_04_playing: GOOD: mid-play; bottom stick+DASH chrome outside field; HUD OK; portrait | BAD: none
CRITIQUE phone_large_05_game_over: GOOD: real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none
CRITIQUE phone_iphone_promax_01_boot: GOOD: title+CTA settled; portrait 736×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_iphone_promax_02_menu: GOOD: stick/DASH copy + swap; panel clean | BAD: none
CRITIQUE phone_iphone_promax_03_mode_select: GOOD: modes+diff+START+hints readable | BAD: none
CRITIQUE phone_iphone_promax_04_playing: GOOD: mid-play; stick+DASH chrome outside field | BAD: none
CRITIQUE phone_iphone_promax_05_game_over: GOOD: real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none
CRITIQUE phone_iphone_promax_landscape_01_boot: GOOD: title+CTA settled landscape 1600×736 emu; no Translate | BAD: none
CRITIQUE phone_iphone_promax_landscape_02_menu: GOOD: stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_iphone_promax_landscape_03_mode_select: GOOD: modes+diff+wide START readable landscape | BAD: none
CRITIQUE phone_iphone_promax_landscape_04_playing: GOOD: PSP left stick + right DASH outside field; HUD OK | BAD: none
CRITIQUE phone_iphone_promax_landscape_05_game_over: GOOD: real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
CRITIQUE phone_samsung_ultra_01_boot: GOOD: title+CTA settled; portrait 720×1600 emu; no Translate | BAD: none
CRITIQUE phone_samsung_ultra_02_menu: GOOD: stick/DASH copy + swap; panel clean | BAD: none
CRITIQUE phone_samsung_ultra_03_mode_select: GOOD: modes+diff+START+hints readable | BAD: none
CRITIQUE phone_samsung_ultra_04_playing: GOOD: mid-play; stick+DASH bottom chrome outside field | BAD: none
CRITIQUE phone_samsung_ultra_05_game_over: GOOD: real **GAME OVER** unclipped SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none

## tablet_portrait + tablet_landscape + tablet_large_portrait
CRITIQUE tablet_portrait_01_boot: GOOD: title RUSTY DASHER + ENTER/click/tap CTA settled; portrait 1200×1600; **blank FIXED**; no Translate | BAD: none (lab multi-tab Chrome chrome OK for full-display emulator path)
CRITIQUE tablet_portrait_02_menu: GOOD: full menu panel — stick/DASH phone copy + swap strip; Best 2; **blank FIXED** | BAD: none
CRITIQUE tablet_portrait_03_mode_select: GOOD: SELECT MODE all 4 modes + NORMAL + green START + touch hints; **blank FIXED** | BAD: none
CRITIQUE tablet_portrait_04_playing: GOOD: mid-play field + stars; bottom stick+DASH chrome outside field; HUD Score/hearts; **blank FIXED** | BAD: none
CRITIQUE tablet_portrait_05_game_over: GOOD: real **GAME OVER** unclipped — CLASSIC/NORMAL Score 0 stats + touch hints; portrait 1200×1600; **force_go FIXED** | BAD: none
CRITIQUE tablet_landscape_01_boot: GOOD: title RUSTY DASHER + CTA settled; landscape 1600×1200 emu; full qa_matrix URL; no Translate | BAD: none (lab multi-tab Chrome chrome OK)
CRITIQUE tablet_landscape_02_menu: GOOD: stick/DASH phone copy + swap; tablet landscape panel | BAD: none
CRITIQUE tablet_landscape_03_mode_select: GOOD: SELECT MODE + START + touch hints tablet landscape | BAD: none
CRITIQUE tablet_landscape_04_playing: GOOD: mid-play; PSP left stick + right DASH outside field; HUD Score/hearts | BAD: none
CRITIQUE tablet_landscape_05_game_over: GOOD: real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + tablet touch hints; force_go | BAD: none
CRITIQUE tablet_large_portrait_01_boot: GOOD: title+CTA settled; portrait 1112×1600 emu; no Translate | BAD: none (lab multi-tab Chrome chrome OK)
CRITIQUE tablet_large_portrait_02_menu: GOOD: stick/DASH copy + swap; large tablet portrait | BAD: none
CRITIQUE tablet_large_portrait_03_mode_select: GOOD: modes+diff+START+hints readable large tablet | BAD: none
CRITIQUE tablet_large_portrait_04_playing: GOOD: mid-play; bottom stick+DASH chrome outside field; HUD OK | BAD: none
CRITIQUE tablet_large_portrait_05_game_over: GOOD: real **GAME OVER** unclipped SURVIVAL/NORMAL Score 0 + tablet touch hints; force_go | BAD: none

## phone_samsung_ultra_landscape + phone_rodin + phone_rodin_chrome + phone_rodin_landscape
CRITIQUE phone_samsung_ultra_landscape_01_boot: GOOD: title+CTA settled landscape 1600×720 emu; full qa_matrix URL; no Translate | BAD: none
CRITIQUE phone_samsung_ultra_landscape_02_menu: GOOD: stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_samsung_ultra_landscape_03_mode_select: GOOD: modes+diff+wide START landscape | BAD: none
CRITIQUE phone_samsung_ultra_landscape_04_playing: GOOD: PSP left stick + right DASH outside field; HUD OK | BAD: none
CRITIQUE phone_samsung_ultra_landscape_05_game_over: GOOD: real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
CRITIQUE phone_rodin_01_boot: GOOD: title+CTA settled; portrait 718×1600 emu (rodin CSS); no Translate | BAD: none
CRITIQUE phone_rodin_02_menu: GOOD: stick/DASH copy + swap; panel clean | BAD: none
CRITIQUE phone_rodin_03_mode_select: GOOD: modes+diff+START+hints readable | BAD: none
CRITIQUE phone_rodin_04_playing: GOOD: mid-play; stick+DASH bottom chrome outside field | BAD: none
CRITIQUE phone_rodin_05_game_over: GOOD: real **GAME OVER** unclipped SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none
CRITIQUE phone_rodin_chrome_01_boot: GOOD: title+CTA settled; chrome-height 868×1600 emu; no Translate; not mid-download | BAD: none
CRITIQUE phone_rodin_chrome_02_menu: GOOD: stick/DASH copy + swap; panel clean at chrome-height | BAD: none
CRITIQUE phone_rodin_chrome_03_mode_select: GOOD: modes+diff+START+hints readable chrome-height | BAD: none
CRITIQUE phone_rodin_chrome_04_playing: GOOD: mid-play; stick+DASH bottom chrome outside field; primary touch-map repro layout OK | BAD: none
CRITIQUE phone_rodin_chrome_05_game_over: GOOD: real **GAME OVER** unclipped SURVIVAL/NORMAL Score 0 + touch hints; force_go | BAD: none
CRITIQUE phone_rodin_landscape_01_boot: GOOD: title+CTA settled landscape 1600×718 emu; no Translate | BAD: none
CRITIQUE phone_rodin_landscape_02_menu: GOOD: stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_rodin_landscape_03_mode_select: GOOD: modes+diff+wide START landscape | BAD: none
CRITIQUE phone_rodin_landscape_04_playing: GOOD: PSP left stick + right DASH outside field; HUD OK | BAD: none
CRITIQUE phone_rodin_landscape_05_game_over: GOOD: real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
