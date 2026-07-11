# Viewport matrix critiques
PRE-PROD: all BAD must be none before push.
Run: 2026-07-11 FIX-LOOP 2 + QUALITY PASS desktop (pid 319115; mtime>=run_start_unix 1783780422; re-reviewed units supersede prior BADs)
NOTE: lines rewritten as each unit re-reviewed on NEW artifacts only.
---

## pipeline batch
CRITIQUE phone_android_01_boot: GOOD: QP re-review mtime 11:34; title RUSTY DASHER + ENTER/click/tap CTA settled (not mid-WASM); portrait 720×1600 emu; no Translate; qa_matrix URL | BAD: none (lab Chrome URL bar OK for full-display emulator path)
CRITIQUE phone_android_02_menu: GOOD: QP re-review mtime 11:34; stick/DASH phone copy + swap strip; Best 3; portrait; no Translate | BAD: none
CRITIQUE phone_android_03_mode_select: GOOD: QP re-review mtime 11:34; SELECT MODE all 4 modes + NORMAL + green START + touch hints; portrait | BAD: none
CRITIQUE phone_android_04_playing: GOOD: QP re-review mtime 11:34; mid-play field + stars/hazards; bottom stick+DASH chrome outside field; Score/hearts HUD; portrait | BAD: none
CRITIQUE phone_android_05_game_over: GOOD: QP re-review mtime 11:35; real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 stats + touch hints; portrait 720×1600; force_go | BAD: none
CRITIQUE phone_android_landscape_01_boot: GOOD: QP re-review mtime 11:35; title RUSTY DASHER + CTA settled; landscape 1600×720; full qa_matrix+qa_go_ms URL; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_02_menu: GOOD: QP re-review mtime 11:35; stick/DASH phone copy + swap strip; landscape panel fits; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_03_mode_select: GOOD: QP re-review mtime 11:35; SELECT MODE all 4 + NORMAL + wide START + touch hints; landscape tight height OK | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_04_playing: GOOD: QP re-review mtime 11:36; mid-play; PSP left stick + right DASH side chrome; Score/hearts HUD; landscape | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_05_game_over: GOOD: QP re-review mtime 11:36; real **GAME OVER** unclipped — CLASSIC/NORMAL Score 0 + touch hints; landscape 1600×720; force_go | BAD: none
CRITIQUE phone_portrait_01_boot: GOOD: QP re-review mtime 11:37; title RUSTY DASHER + CTA settled; portrait 738×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_portrait_02_menu: GOOD: QP re-review mtime 11:37; stick/DASH phone copy + swap; panel centered; portrait | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: QP re-review mtime 11:37; modes+difficulty+START + touch hints readable; portrait | BAD: none
CRITIQUE phone_portrait_04_playing: GOOD: QP re-review mtime 11:37; mid-play field; bottom stick+DASH chrome outside field; HUD legible; portrait | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: QP re-review mtime 11:38; real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 + touch hints; portrait; force_go | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: QP re-review mtime 11:38; title+CTA settled; landscape 1600×738 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_landscape_02_menu: GOOD: QP re-review mtime 11:39; stick/DASH copy + swap; panel fits landscape | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: QP re-review mtime 11:39; modes+diff+wide START; hints readable landscape | BAD: none
CRITIQUE phone_landscape_04_playing: GOOD: QP re-review mtime 11:39; PSP left stick + right DASH outside field; HUD score/hearts; mid-play; landscape | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: QP re-review mtime 11:40; real **GAME OVER** unclipped — CLASSIC/NORMAL + touch hints; landscape; force_go | BAD: none

## qhd + 4k
CRITIQUE qhd_01_boot: GOOD: title+CTA centered readable at QHD NEW quality-pass | BAD: none
CRITIQUE qhd_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome NEW | BAD: none
CRITIQUE qhd_03_mode_select: GOOD: modes+difficulty readable NEW | BAD: none
CRITIQUE qhd_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible NEW | BAD: none
CRITIQUE qhd_05_game_over: GOOD: real GAME OVER unclipped CLASSIC/NORMAL Score 1; desktop ENTER/SPACE/ESC hints NEW | BAD: none
CRITIQUE 4k_01_boot: GOOD: title+CTA centered readable at 4K NEW quality-pass (UI sparse OK for CSS 4K) | BAD: none
CRITIQUE 4k_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome NEW | BAD: none
CRITIQUE 4k_03_mode_select: GOOD: modes+difficulty readable at 4K NEW | BAD: none
CRITIQUE 4k_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible NEW | BAD: none
CRITIQUE 4k_05_game_over: GOOD: real NEW HIGH SCORE! unclipped Score 4; desktop ENTER/SPACE/ESC hints NEW | BAD: none

## desktop batch (laptop_hd laptop_scaled laptop_720 1080p)
CRITIQUE laptop_hd_01_boot: GOOD: title RUSTY DASHER + ENTER/click/tap CTA centered readable at 1366×768 NEW quality-pass visual A6 (post-recapture matrix OK) | BAD: none
CRITIQUE laptop_hd_02_menu: GOOD: desktop WASD/arrows + SPACE dash + mouse point-to-move/right-click copy; no stick/DASH chrome NEW visual A6 | BAD: none
CRITIQUE laptop_hd_03_mode_select: GOOD: SELECT MODE all 4 modes + NORMAL + keyboard hints readable NEW visual A6 | BAD: none
CRITIQUE laptop_hd_04_playing: GOOD: full field no stick chrome; Score/hearts/HUD; Dash READY; player+stars+hazard NEW visual A6 (post-recapture) | BAD: none
CRITIQUE laptop_hd_05_game_over: GOOD: real GAME OVER unclipped CLASSIC/NORMAL Score 0 + desktop ENTER/SPACE/ESC hints NEW visual A6 (post-recapture) | BAD: none
CRITIQUE laptop_scaled_01_boot: GOOD: title+CTA centered readable at 1536×864 NEW quality-pass | BAD: none
CRITIQUE laptop_scaled_02_menu: GOOD: desktop WASD/mouse control copy; no stick chrome NEW | BAD: none
CRITIQUE laptop_scaled_03_mode_select: GOOD: modes+difficulty+keyboard hints readable NEW | BAD: none
CRITIQUE laptop_scaled_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible NEW | BAD: none
CRITIQUE laptop_scaled_05_game_over: GOOD: real NEW HIGH SCORE! unclipped Score 4; desktop keyboard hints NEW | BAD: none
CRITIQUE laptop_720_01_boot: GOOD: title+CTA centered readable at 1280×720 NEW quality-pass | BAD: none
CRITIQUE laptop_720_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome NEW | BAD: none
CRITIQUE laptop_720_03_mode_select: GOOD: modes+difficulty+keyboard hints readable NEW | BAD: none
CRITIQUE laptop_720_04_playing: GOOD: full field no stick chrome; Dash READY; HUD OK NEW | BAD: none
CRITIQUE laptop_720_05_game_over: GOOD: real GAME OVER unclipped CLASSIC/NORMAL Score 1; desktop ENTER/SPACE/ESC hints NEW | BAD: none
CRITIQUE 1080p_01_boot: GOOD: title+CTA centered readable at 1920×1080 NEW quality-pass | BAD: none
CRITIQUE 1080p_02_menu: GOOD: desktop WASD/mouse copy; no stick chrome NEW | BAD: none
CRITIQUE 1080p_03_mode_select: GOOD: modes+difficulty readable NEW | BAD: none
CRITIQUE 1080p_04_playing: GOOD: full field no stick chrome; Dash READY; HUD legible NEW | BAD: none
CRITIQUE 1080p_05_game_over: GOOD: real GAME OVER unclipped CLASSIC/NORMAL Score 2; desktop keyboard hints NEW | BAD: none
CRITIQUE phone_large_01_boot: GOOD: QP re-review mtime 11:41; title+CTA settled; portrait 738×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_large_02_menu: GOOD: QP re-review mtime 11:41; stick/DASH phone copy + swap strip; layout clean; portrait | BAD: none
CRITIQUE phone_large_03_mode_select: GOOD: QP re-review mtime 11:41; modes+diff+START+hints readable; portrait | BAD: none
CRITIQUE phone_large_04_playing: GOOD: QP re-review mtime 11:41; mid-play; bottom stick+DASH chrome outside field; HUD OK; portrait | BAD: none
CRITIQUE phone_large_05_game_over: GOOD: QP re-review mtime 11:41; real **GAME OVER** unclipped; force_go | BAD: none
CRITIQUE phone_iphone_promax_01_boot: GOOD: QP re-review mtime 11:42; title RUSTY DASHER + ENTER/click/tap CTA settled; portrait 736×1600 emu; no Translate; qa_matrix URL | BAD: none (lab Chrome URL bar OK for full-display emulator path)
CRITIQUE phone_iphone_promax_02_menu: GOOD: QP re-review mtime 11:42; stick/DASH phone copy + swap strip; Best 3; portrait; no Translate | BAD: none
CRITIQUE phone_iphone_promax_03_mode_select: GOOD: QP re-review mtime 11:42; SELECT MODE all 4 modes + NORMAL + green START + touch hints; portrait | BAD: none
CRITIQUE phone_iphone_promax_04_playing: GOOD: QP re-review mtime 11:43; mid-play field + stars/hazards; bottom stick+DASH chrome outside field; Score/hearts HUD; portrait | BAD: none
CRITIQUE phone_iphone_promax_05_game_over: GOOD: QP re-review mtime 11:43; real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 2 + touch hints; portrait 736×1600; force_go | BAD: none
CRITIQUE phone_iphone_promax_landscape_01_boot: GOOD: QP re-review mtime 11:44; title+CTA settled; landscape 1600×736 emu; no Translate; full qa_matrix URL | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_iphone_promax_landscape_02_menu: GOOD: QP re-review mtime 11:44; stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_iphone_promax_landscape_03_mode_select: GOOD: QP re-review mtime 11:44; modes+diff+wide START readable landscape | BAD: none
CRITIQUE phone_iphone_promax_landscape_04_playing: GOOD: QP re-review mtime 11:44; mid-play; PSP left stick + right DASH outside field; Score/hearts HUD | BAD: none
CRITIQUE phone_iphone_promax_landscape_05_game_over: GOOD: QP re-review mtime 11:45; real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
CRITIQUE phone_samsung_ultra_01_boot: GOOD: QP re-review mtime 11:46; title+CTA settled; portrait 720×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_samsung_ultra_02_menu: GOOD: QP re-review mtime 11:46; stick/DASH copy + swap; panel clean | BAD: none
CRITIQUE phone_samsung_ultra_03_mode_select: GOOD: QP re-review mtime 11:46; modes+diff+START+hints readable | BAD: none
CRITIQUE phone_samsung_ultra_04_playing: GOOD: QP re-review mtime 11:46; mid-play; bottom stick+DASH chrome outside field; Score/hearts HUD | BAD: none
CRITIQUE phone_samsung_ultra_05_game_over: GOOD: QP re-review mtime 11:46; real **GAME OVER** unclipped SURVIVAL/NORMAL Score 0 + touch hints; force_go | BAD: none

## tablet_portrait + tablet_landscape + tablet_large_portrait
CRITIQUE tablet_portrait_01_boot: GOOD: QP re-review mtime 11:53; title+CTA settled; portrait 1200×1600; not blank; no Translate | BAD: none (lab multi-tab Chrome chrome OK for full-display emulator path)
CRITIQUE tablet_portrait_02_menu: GOOD: QP re-review mtime 11:53; full menu panel — stick/DASH phone copy + swap strip; Best 3 | BAD: none
CRITIQUE tablet_portrait_03_mode_select: GOOD: QP re-review mtime 11:54; SELECT MODE all 4 modes + NORMAL + green START + touch hints | BAD: none
CRITIQUE tablet_portrait_04_playing: GOOD: QP re-review mtime 11:54; mid-play field + stars; bottom stick+DASH chrome outside field; HUD Score/hearts | BAD: none
CRITIQUE tablet_portrait_05_game_over: GOOD: QP re-review mtime 11:54; real **GAME OVER** unclipped — CLASSIC/NORMAL Score 1 + tablet touch hints (play again / two fingers menu); force_go | BAD: none
CRITIQUE tablet_landscape_01_boot: GOOD: QP re-review mtime 11:55; title+CTA settled; landscape 1600×1200 emu; no Translate | BAD: none (lab multi-tab Chrome chrome OK)
CRITIQUE tablet_landscape_02_menu: GOOD: QP re-review mtime 11:55; stick/DASH phone copy + swap; tablet landscape panel | BAD: none
CRITIQUE tablet_landscape_03_mode_select: GOOD: QP re-review mtime 11:55; SELECT MODE + START + touch hints tablet landscape | BAD: none
CRITIQUE tablet_landscape_04_playing: GOOD: QP re-review mtime 11:55; mid-play; PSP left stick + right DASH outside field; HUD Score/hearts | BAD: none
CRITIQUE tablet_landscape_05_game_over: GOOD: QP re-review mtime 11:56; real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + tablet touch hints; force_go | BAD: none
CRITIQUE tablet_large_portrait_01_boot: GOOD: QP re-review mtime 11:56; title+CTA settled; portrait 1112×1600 emu; no Translate | BAD: none (lab multi-tab Chrome chrome OK)
CRITIQUE tablet_large_portrait_02_menu: GOOD: QP re-review mtime 11:56; stick/DASH copy + swap; large tablet portrait | BAD: none
CRITIQUE tablet_large_portrait_03_mode_select: GOOD: QP re-review mtime 11:57; modes+diff+START+hints readable large tablet | BAD: none
CRITIQUE tablet_large_portrait_04_playing: GOOD: QP re-review mtime 11:57; mid-play; bottom stick+DASH chrome outside field; HUD OK | BAD: none
CRITIQUE tablet_large_portrait_05_game_over: GOOD: QP re-review mtime 11:57; real **GAME OVER** unclipped SURVIVAL/NORMAL Score 0 + tablet touch hints; force_go | BAD: none

## phone_samsung_ultra_landscape + phone_rodin + phone_rodin_chrome + phone_rodin_landscape
CRITIQUE phone_samsung_ultra_landscape_01_boot: GOOD: QP re-review mtime 11:47; title+CTA settled landscape 1600×720 emu; no Translate; full qa_matrix URL | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_samsung_ultra_landscape_02_menu: GOOD: QP re-review mtime 11:47; stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_samsung_ultra_landscape_03_mode_select: GOOD: QP re-review mtime 11:47; modes+diff+wide START landscape; touch hints tight on START edge but readable | BAD: none
CRITIQUE phone_samsung_ultra_landscape_04_playing: GOOD: QP re-review mtime 11:47; mid-play; PSP left stick + right DASH outside field; HUD OK | BAD: none
CRITIQUE phone_samsung_ultra_landscape_05_game_over: GOOD: QP re-review mtime 11:48; real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
CRITIQUE phone_rodin_01_boot: GOOD: QP re-review mtime 11:49; title+CTA settled; portrait 718×1600 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_rodin_02_menu: GOOD: QP re-review mtime 11:49; stick/DASH copy + swap; panel clean | BAD: none
CRITIQUE phone_rodin_03_mode_select: GOOD: QP re-review mtime 11:49; modes+diff+START+hints readable | BAD: none
CRITIQUE phone_rodin_04_playing: GOOD: QP re-review mtime 11:49; mid-play; bottom stick+DASH chrome outside field; HUD OK | BAD: none
CRITIQUE phone_rodin_05_game_over: GOOD: QP re-review mtime 11:50; real **GAME OVER** unclipped SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none
CRITIQUE phone_rodin_chrome_01_boot: GOOD: QP re-review mtime 11:50; title+CTA settled; chrome-height 868×1600 emu; no Translate; not mid-download | BAD: none
CRITIQUE phone_rodin_chrome_02_menu: GOOD: QP re-review mtime 11:51; stick/DASH copy + swap; panel clean at chrome-height | BAD: none
CRITIQUE phone_rodin_chrome_03_mode_select: GOOD: QP re-review mtime 11:51; modes+diff+START+hints readable chrome-height | BAD: none
CRITIQUE phone_rodin_chrome_04_playing: GOOD: QP re-review mtime 11:51; mid-play; stick+DASH bottom chrome outside field; primary touch-map layout OK | BAD: none
CRITIQUE phone_rodin_chrome_05_game_over: GOOD: QP re-review mtime 11:51; real **GAME OVER** unclipped SURVIVAL/NORMAL Score 1 + touch hints; force_go | BAD: none
CRITIQUE phone_rodin_landscape_01_boot: GOOD: QP re-review mtime 11:52; title+CTA settled landscape 1600×718 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_rodin_landscape_02_menu: GOOD: QP re-review mtime 11:52; stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_rodin_landscape_03_mode_select: GOOD: QP re-review mtime 11:52; modes+diff+wide START landscape; touch hints tight on START edge but readable | BAD: none
CRITIQUE phone_rodin_landscape_04_playing: GOOD: QP re-review mtime 11:52; mid-play; PSP left stick + right DASH outside field; HUD OK | BAD: none
CRITIQUE phone_rodin_landscape_05_game_over: GOOD: QP re-review mtime 11:53; real **GAME OVER** unclipped CLASSIC/NORMAL Score 0 + touch hints; landscape; force_go | BAD: none
