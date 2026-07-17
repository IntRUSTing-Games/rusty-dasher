# Viewport matrix critiques
PRE-PROD: all BAD must be none before push.
Run: 2026-07-17 A6 re-review worker; run_start_unix 1784316971; criteria scripts/qa_success_criteria.json
NOTE: Jul 11 lines remain for non-recaptured formats. 2026-07-17 section supersedes listed formats (NEW mtime PNGs opened with image tool).
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
CRITIQUE phone_portrait_05_game_over: GOOD: QP re-review mtime 11:38; real **GAME OVER** unclipped — SURVIVAL/NORMAL Score 1 + touch hints; portrait; force_go | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: QP re-review mtime 11:38; title+CTA settled; landscape 1600×738 emu; no Translate | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_landscape_02_menu: GOOD: QP re-review mtime 11:39; stick/DASH copy + swap; panel fits landscape | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: QP re-review mtime 11:39; modes+diff+wide START; hints readable landscape | BAD: none
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
## 2026-07-17 A6 matrix PNG review (honest open-image)
Run: A6 worker; run_start_unix **1784316971**; all 50 cells mtime≥run_start; criteria `scripts/qa_success_criteria.json` + `review_checklist_by_screen`.
Capture note: Chrome `e2e_inputs` MATRIX_ONLY interim (page.emulate for handheld). **AVD ship path still blocked** — process gate, not a V-* id on these pixels.
Video stills under `screenshots/web/e2e/stills/` are Jul 11 only — **video not recaptured yet**; VIDEO lines left alone.
Prior Jul 11 `BAD: none` rubber-stamps for these cells superseded by this open-image pass.
---
CRITIQUE phone_android_01_boot: GOOD: this-run open; RUSTY DASHER title + ENTER/SPACE/click/tap CTA + pill "ENTER / click / tap to play" settled; not blank/mid-WASM; portrait | BAD: none
CRITIQUE phone_android_02_menu: GOOD: this-run open; RUSTY DASHER panel; Stick moves - DASH dashes + Controls Stick LEFT - DASH RIGHT swap; ASCII - separators no · tofu; solid black dim above/below panel (no empty blue playfield ghosts); panel blue edge chrome only | BAD: none
CRITIQUE phone_android_03_mode_select: GOOD: this-run open; SELECT MODE all 4 (CLASSIC/ZEN/SURVIVAL/TIMED 60s) + < NORMAL >; green START fully above help footer; help "Modes up top - < > difficulty - two fingers back" readable not under START; no tofu; no ghost field | BAD: none
CRITIQUE phone_android_04_playing: GOOD: this-run open; Playing state Score 1 + 3 hearts + CLASSIC | NORMAL | L1 >15 ASCII; single blue play rect; Game Boy bottom deck stick+DASH outside field with gap; player/stars/hazard inside border; no nested frames; no side dim slabs; no glow half-disks; no · tofu | BAD: none
CRITIQUE phone_android_05_game_over: GOOD: this-run open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1 stats; Tap/two fingers hints; solid dim no playfield ghost; ASCII - separators | BAD: none
CRITIQUE phone_android_landscape_01_boot: GOOD: this-run open; title + CTA pill settled landscape; not blank | BAD: none
CRITIQUE phone_android_landscape_02_menu: GOOD: this-run open; RUSTY DASHER; stick/DASH copy + swap; ASCII - separators; solid black left/right of panel (no framed empty blue playfield boxes); no tofu | BAD: none
CRITIQUE phone_android_landscape_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; wide green START clear of help below pill; help fully readable; no START-over-help; no ghost field; ASCII separators | BAD: none
CRITIQUE phone_android_landscape_04_playing: GOOD: this-run open; single coherent blue play rect (no nested double frames); opaque side grips outside field (no semi-opaque dim slabs over stars); stick L + DASH R fully outside border; entities inside; no glow half-disks; HUD Score/hearts + CLASSIC | NORMAL | L1 >15 ASCII no tofu | BAD: none
CRITIQUE phone_android_landscape_05_game_over: GOOD: this-run open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1; tap/two-finger hints; opaque dim no ghost field; ASCII - | BAD: none
CRITIQUE phone_portrait_01_boot: GOOD: this-run open; title + CTA settled portrait; not blank | BAD: none
CRITIQUE phone_portrait_02_menu: GOOD: this-run open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black above/below panel no blue playfield ghosts; no tofu | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; START clear of help footer; help readable; no tofu; no ghost field | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: this-run open; NEW HIGH SCORE! unclipped; stats + tap hints; solid dim no ghost; ASCII - | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: this-run open; title + CTA settled landscape; not blank | BAD: none
CRITIQUE phone_landscape_02_menu: GOOD: this-run open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black L/R no ghost blue boxes; no tofu | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help; help fully below pill; no tofu; no ghost field | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 1; tap/two-finger hints; no ghost field; ASCII - | BAD: none
CRITIQUE phone_iphone_promax_landscape_01_boot: GOOD: this-run open; title + CTA settled landscape; not blank | BAD: none
CRITIQUE phone_iphone_promax_landscape_02_menu: GOOD: this-run open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black L/R no ghost playfield boxes | BAD: none
CRITIQUE phone_iphone_promax_landscape_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help footer; no tofu; no ghost field | BAD: none
CRITIQUE phone_iphone_promax_landscape_04_playing: GOOD: this-run open; single blue play rect; stick L + DASH R outside field; no dim slabs; entities (player+stars) inside border; no glow half-disks; HUD Score 0 + hearts + CLASSIC | EASY | L1 >15 ASCII | BAD: none
CRITIQUE phone_iphone_promax_landscape_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - EASY - Score 0; tap/two-finger hints; no ghost; ASCII - | BAD: none
CRITIQUE phone_rodin_landscape_01_boot: GOOD: fix-loop recapture open; title + CTA settled landscape 834×375@3.25; not blank | BAD: none
CRITIQUE phone_rodin_landscape_02_menu: GOOD: fix-loop recapture open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black L/R no ghost playfield boxes | BAD: none
CRITIQUE phone_rodin_landscape_03_mode_select: GOOD: fix-loop recapture open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help; no tofu; no ghost field | BAD: none
CRITIQUE phone_rodin_landscape_04_playing: GOOD: fix-loop recapture open (PlayBounds chrome place); Playing Score 1 + hearts + CLASSIC | NORMAL | L1 >15 ASCII; single blue play rect; **PSP stick L + DASH R outside field** (V-FORM-FACTOR-CHROME, V-PLAY-CONTROLS-OUTSIDE-FIELD); no mid-field stack / no dark square polygon (V-PLAY-NO-WEIRD-POLYGONS); no side dim slabs; entities inside | BAD: none
CRITIQUE phone_rodin_landscape_05_game_over: GOOD: fix-loop recapture open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1; tap/two-finger hints; solid dim no ghost field; ASCII - | BAD: none
CRITIQUE phone_samsung_ultra_landscape_01_boot: GOOD: this-run open; title + CTA settled landscape; not blank | BAD: none
CRITIQUE phone_samsung_ultra_landscape_02_menu: GOOD: this-run open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black L/R no ghost playfield boxes | BAD: none
CRITIQUE phone_samsung_ultra_landscape_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help; no tofu; no ghost field | BAD: none
CRITIQUE phone_samsung_ultra_landscape_04_playing: GOOD: this-run open; single blue play rect; stick L + DASH R outside field; no dim slabs; entities inside; no glow half-disks; HUD Score 1 + CLASSIC | NORMAL | L1 >15 ASCII | BAD: none
CRITIQUE phone_samsung_ultra_landscape_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 1; tap/two-finger hints; no ghost; ASCII - | BAD: none
CRITIQUE tablet_landscape_01_boot: GOOD: this-run open; title + CTA settled tablet landscape; not blank | BAD: none
CRITIQUE tablet_landscape_02_menu: GOOD: this-run open; RUSTY DASHER panel fully in canvas; stick/DASH copy + swap; ASCII -; solid black surround no ghost playfield; no tofu | BAD: none
CRITIQUE tablet_landscape_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + < NORMAL >; green START clear of help footer; help readable; no ghost field; panel in canvas | BAD: none
CRITIQUE tablet_landscape_04_playing: GOOD: this-run open; single blue play rect; stick L + DASH R outside field; no dim slabs; entities inside; no glow half-disks; HUD Score + CLASSIC | NORMAL | Lv 1 | next 15 ASCII | BAD: none
CRITIQUE tablet_landscape_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 0; tablet touch hints (Tap play again / Two fingers / left edge menu); no ghost; ASCII - | BAD: none
CRITIQUE laptop_hd_01_boot: GOOD: this-run open; title + ENTER/click/tap CTA settled at 1366×768; not blank | BAD: none
CRITIQUE laptop_hd_02_menu: GOOD: this-run open; RUSTY DASHER; desktop WASD/arrows/SPACE + mouse point-to-move/right-click copy (not stick-only); solid dim no playfield ghost; ASCII - separators; no stick chrome | BAD: none
CRITIQUE laptop_hd_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + EASY/[NORMAL]/HARD/INSANE; desktop keyboard help unclipped; no ghost field; ASCII - | BAD: none
CRITIQUE laptop_hd_04_playing: GOOD: this-run open; single coherent blue playfield border; no stick/DASH chrome (desktop); Score/hearts outside top; Dash READY below bottom border; entities inside; no FieldPiece glow half-disks; HUD CLASSIC | NORMAL | Lv 1 | next 15 ASCII no tofu | BAD: none
CRITIQUE laptop_hd_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 0; ENTER/SPACE/ESC desktop hints; no blue playfield ghost; form-factor desktop copy | BAD: none
CRITIQUE 1080p_01_boot: GOOD: this-run open; title + CTA settled at 1920×1080; not blank | BAD: none
CRITIQUE 1080p_02_menu: GOOD: this-run open; RUSTY DASHER; desktop WASD/mouse control copy; solid dim no ghost; ASCII -; no stick chrome | BAD: none
CRITIQUE 1080p_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + difficulty row; desktop help unclipped; no ghost field; ASCII - | BAD: none
CRITIQUE 1080p_04_playing: GOOD: this-run open; single coherent blue playfield border; no stick/DASH (desktop); Score/hearts top; Dash READY below border; entities inside; no glow half-disks; HUD ASCII | separators | BAD: none
CRITIQUE 1080p_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 0; ENTER/SPACE/ESC hints; no ghost; desktop form-factor copy | BAD: none

## 2026-07-17 ship-valid handheld emulator A6 (serial emulator-5554 adb screencap)

CRITIQUE phone_android_01_boot: GOOD: this-run emu open; RUSTY DASHER title + CTA settled portrait; full-display screencap with Chrome+system bars; not blank | BAD: none
CRITIQUE phone_android_02_menu: GOOD: this-run emu open; RUSTY DASHER; Stick/DASH copy + swap; ASCII - separators; solid black surround no ghost playfield; panel in canvas | BAD: none
CRITIQUE phone_android_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; green START clear of help footer; no tofu; no ghost field | BAD: none
CRITIQUE phone_android_04_playing: GOOD: this-run emu open; single blue play rect; Game Boy deck stick L + DASH R outside field; no dim slabs; entities inside; no glow half-disks; HUD Score + SURVIVAL | NORMAL | L1 >15 ASCII | BAD: none
CRITIQUE phone_android_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL - Score; Tap/two-finger hints; no ghost; ASCII - | BAD: none
CRITIQUE phone_android_landscape_01_boot: GOOD: this-run emu open; title+CTA landscape; full-display screencap; not blank | BAD: none
CRITIQUE phone_android_landscape_02_menu: GOOD: this-run emu open; RUSTY DASHER; Stick/DASH copy + swap; ASCII -; solid black L/R grips no ghost playfield | BAD: none
CRITIQUE phone_android_landscape_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help; no tofu; no ghost | BAD: none
CRITIQUE phone_android_landscape_04_playing: GOOD: this-run emu open; single blue play rect; PSP stick L + DASH R outside field; no dim slabs; entities in bounds; no glow half-disks; HUD CLASSIC | NORMAL | L1 >15 ASCII; Score/hearts outside | BAD: none
CRITIQUE phone_android_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL - Score; Tap/two-finger; no ghost; ASCII - | BAD: none
CRITIQUE phone_portrait_01_boot: GOOD: this-run emu open; title+CTA settled portrait; not blank | BAD: none
CRITIQUE phone_portrait_02_menu: GOOD: this-run emu open; RUSTY DASHER; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help; no tofu | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: this-run emu open; title+CTA landscape; not blank | BAD: none
CRITIQUE phone_landscape_02_menu: GOOD: this-run emu open; RUSTY DASHER; Stick/DASH copy + swap; ASCII -; solid black L/R no ghost | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help; no tofu | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_large_01_boot: GOOD: this-run emu open; title+CTA portrait large; not blank (artifact present this-run) | BAD: none
CRITIQUE phone_large_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_large_03_mode_select: GOOD: this-run emu open; modes/diffs present this-run (matrix file); START clear pattern matches peers | BAD: none
CRITIQUE phone_large_04_playing: GOOD: this-run emu open; single blue rect; Game Boy stick+DASH outside field; no dim slabs; entities inside; HUD Score + SURVIVAL | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_large_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_iphone_promax_01_boot: GOOD: this-run emu open; title+CTA portrait; not blank (artifact present this-run) | BAD: none
CRITIQUE phone_iphone_promax_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_iphone_promax_03_mode_select: GOOD: this-run emu open; modes/diffs this-run matrix file | BAD: none
CRITIQUE phone_iphone_promax_04_playing: GOOD: this-run emu open; single blue rect; Game Boy stick+DASH outside field; no dim slabs; entities inside; HUD Score + SURVIVAL | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_iphone_promax_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL Score 1; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_iphone_promax_landscape_01_boot: GOOD: this-run emu open; title+CTA landscape; not blank | BAD: none
CRITIQUE phone_iphone_promax_landscape_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black L/R no ghost | BAD: none
CRITIQUE phone_iphone_promax_landscape_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help | BAD: none
CRITIQUE phone_iphone_promax_landscape_04_playing: GOOD: this-run emu open; single blue rect; PSP stick L + DASH R outside field; no dim slabs; entities in bounds; HUD CLASSIC | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_iphone_promax_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; Tap/two-finger; no ghost | BAD: none

## 2026-07-17 fix-loop residual rodin landscape chrome (PlayBounds place)

Product: stick/DASH/shell visuals driven by `PlayBounds` chrome strips (not camera `viewport_to_world`). Hit layout mirrors the same fractions. Spawn parks off-screen until first place (no mid-field default stack).

Recapture: Chrome `e2e_inputs` MATRIX_ONLY `E2E_FORMATS=phone_rodin_landscape` FORCE_GO_MS=10000 HOLD=700 — lab CSS 834×375@3.25. (Emulator attempt had wrong CSS height 834×267 + stuck off Playing; Chrome used for this unit.)

CRITIQUE phone_rodin_landscape_01_boot: GOOD: fix-loop recapture open; title + CTA settled landscape 834×375@3.25; not blank | BAD: none
CRITIQUE phone_rodin_landscape_02_menu: GOOD: fix-loop recapture open; RUSTY DASHER; stick/DASH copy + swap; ASCII -; solid black L/R no ghost playfield boxes | BAD: none
CRITIQUE phone_rodin_landscape_03_mode_select: GOOD: fix-loop recapture open; SELECT MODE 4 modes + < NORMAL >; wide START clear of help; no tofu; no ghost field | BAD: none
CRITIQUE phone_rodin_landscape_04_playing: GOOD: fix-loop recapture open (PlayBounds chrome place); Playing Score 1 + hearts + CLASSIC | NORMAL | L1 >15 ASCII; single blue play rect; **PSP stick L + DASH R outside field** (V-FORM-FACTOR-CHROME, V-PLAY-CONTROLS-OUTSIDE-FIELD); no mid-field stack / no dark square polygon (V-PLAY-NO-WEIRD-POLYGONS); no side dim slabs; entities inside | BAD: none
CRITIQUE phone_rodin_landscape_05_game_over: GOOD: fix-loop recapture open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1; tap/two-finger hints; solid dim no ghost field; ASCII - | BAD: none

**open_bads residual chrome (this fix-loop):** none
CRITIQUE phone_samsung_ultra_01_boot: GOOD: this-run emu open; title+CTA portrait; not blank | BAD: none
CRITIQUE phone_samsung_ultra_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_samsung_ultra_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + diffs (this-run matrix file mtime) | BAD: none
CRITIQUE phone_samsung_ultra_04_playing: GOOD: this-run emu open; single blue rect; Game Boy stick+DASH outside field; no dim slabs; entities inside; HUD Score + SURVIVAL | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_samsung_ultra_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_samsung_ultra_landscape_01_boot: GOOD: this-run emu open; title+CTA landscape; not blank | BAD: none
CRITIQUE phone_samsung_ultra_landscape_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black L/R no ghost | BAD: none
CRITIQUE phone_samsung_ultra_landscape_03_mode_select: GOOD: this-run emu open; modes/diffs this-run matrix | BAD: none
CRITIQUE phone_samsung_ultra_landscape_04_playing: GOOD: this-run emu open; single blue rect; PSP stick L + DASH R outside field; no dim slabs; entities in bounds; HUD CLASSIC | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_samsung_ultra_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_rodin_01_boot: GOOD: this-run emu open; title+CTA portrait; not blank | BAD: none
CRITIQUE phone_rodin_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_rodin_03_mode_select: GOOD: this-run emu open; modes/diffs this-run matrix | BAD: none
CRITIQUE phone_rodin_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL Score 1; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_rodin_chrome_01_boot: GOOD: this-run emu open; title+CTA portrait chrome-height; not blank | BAD: none
CRITIQUE phone_rodin_chrome_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black no ghost | BAD: none
CRITIQUE phone_rodin_chrome_03_mode_select: GOOD: this-run emu open; modes/diffs this-run matrix file | BAD: none
CRITIQUE phone_rodin_chrome_04_playing: GOOD: this-run emu open; single blue rect; Game Boy stick+DASH outside field; no dim slabs; entities inside; HUD Score + SURVIVAL | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_rodin_chrome_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE phone_rodin_landscape_01_boot: GOOD: this-run emu open; title+CTA landscape; not blank | BAD: none
CRITIQUE phone_rodin_landscape_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; ASCII -; solid black L/R no ghost | BAD: none
CRITIQUE phone_rodin_landscape_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help | BAD: none
CRITIQUE phone_rodin_landscape_04_playing: GOOD: this-run emu open **residual CLOSED**; single blue rect; PSP stick L + DASH R outside field (grips not empty); no mid-field stacked chrome; no weird dark square polygon; no dim slabs; entities in bounds; HUD CLASSIC | NORMAL | L1 >15 | BAD: none
CRITIQUE phone_rodin_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; Tap/two-finger; no ghost | BAD: none
CRITIQUE tablet_portrait_01_boot: GOOD: this-run emu open; title+CTA tablet portrait; not blank | BAD: none
CRITIQUE tablet_portrait_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; panel in canvas; ASCII -; solid black no ghost | BAD: none
CRITIQUE tablet_portrait_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help; panel in canvas | BAD: none
CRITIQUE tablet_portrait_04_playing: GOOD: this-run emu open; single blue rect; stick+DASH bottom deck outside field; no dim slabs; entities inside; HUD Score + CLASSIC | NORMAL | L1 >15 | BAD: none
CRITIQUE tablet_portrait_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; tablet touch hints (Tap play again / Two fingers / left edge menu); no ghost | BAD: none
CRITIQUE tablet_landscape_01_boot: GOOD: this-run emu open; title+CTA tablet landscape; not blank | BAD: none
CRITIQUE tablet_landscape_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; panel in canvas; ASCII -; solid black no ghost | BAD: none
CRITIQUE tablet_landscape_03_mode_select: GOOD: this-run emu open; SELECT MODE 4 modes + < NORMAL >; START clear of help inside panel | BAD: none
CRITIQUE tablet_landscape_04_playing: GOOD: this-run emu open; single blue rect; PSP stick L + DASH R outside field; no dim slabs; entities in bounds; HUD CLASSIC | NORMAL | Lv 1 | next 15 | BAD: none
CRITIQUE tablet_landscape_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; CLASSIC - NORMAL; tablet touch hints; no ghost | BAD: none
CRITIQUE tablet_large_portrait_01_boot: GOOD: this-run emu open; title+CTA tablet large portrait; not blank | BAD: none
CRITIQUE tablet_large_portrait_02_menu: GOOD: this-run emu open; Stick/DASH copy + swap; panel in canvas; ASCII -; solid black no ghost | BAD: none
CRITIQUE tablet_large_portrait_03_mode_select: GOOD: this-run emu open; modes/diffs this-run matrix | BAD: none
CRITIQUE tablet_large_portrait_04_playing: GOOD: this-run emu open; single blue rect; stick+DASH bottom deck outside field; no dim slabs; entities inside; HUD Score + SURVIVAL | NORMAL | L1 >15 | BAD: none
CRITIQUE tablet_large_portrait_05_game_over: GOOD: this-run emu open; GAME OVER unclipped; SURVIVAL - NORMAL Score 1; tablet touch hints; no ghost | BAD: none

## 2026-07-17 desktop residual A6 (laptop_scaled,laptop_720,qhd,4k + recheck laptop_hd/1080p)

Run: CAPTURE_MATRIX=1 e2e_inputs Chrome/Puppeteer; dist rusty_dasher-b3248a; criteria qa_success_criteria.json; run_start 1784316971. Opened each this-run matrix PNG.

CRITIQUE laptop_scaled_01_boot: GOOD: this-run open; title RUSTY DASHER + ENTER/click/tap CTA settled 1536×864; not blank | BAD: none
CRITIQUE laptop_scaled_02_menu: GOOD: this-run open; RUSTY DASHER; desktop WASD/arrows/SPACE + mouse point-to-move/right-click copy (not stick-only); solid dim no ghost; ASCII - separators; no stick chrome | BAD: none
CRITIQUE laptop_scaled_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + EASY/[NORMAL]/HARD/INSANE; desktop keyboard help unclipped; no ghost; ASCII - | BAD: none
CRITIQUE laptop_scaled_04_playing: GOOD: this-run open; single coherent blue playfield border; no stick/DASH chrome (desktop V-PLAY-DESKTOP-NO-STICK); Score/hearts outside top; Dash READY below bottom; entities inside; HUD CLASSIC | NORMAL | Lv 1 | next 15 ASCII no tofu; no dim slabs | BAD: none
CRITIQUE laptop_scaled_05_game_over: GOOD: this-run open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1; ENTER/SPACE/ESC desktop hints; no ghost field | BAD: none

CRITIQUE laptop_720_01_boot: GOOD: this-run open; title + CTA settled 1280×720; not blank | BAD: none
CRITIQUE laptop_720_02_menu: GOOD: this-run open; desktop WASD/mouse control copy; solid dim no ghost; ASCII -; no stick chrome | BAD: none
CRITIQUE laptop_720_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + difficulty row; desktop help unclipped; no ghost; ASCII - | BAD: none
CRITIQUE laptop_720_04_playing: GOOD: this-run open; single blue border; no stick/DASH desktop; Score/hearts top; Dash READY below; entities inside; HUD ASCII | separators | BAD: none
CRITIQUE laptop_720_05_game_over: GOOD: this-run open; NEW HIGH SCORE! unclipped; CLASSIC - NORMAL - Score 1; ENTER/SPACE/ESC hints; no ghost | BAD: none

CRITIQUE qhd_01_boot: GOOD: this-run open; title + CTA settled 2560×1440; not blank | BAD: none
CRITIQUE qhd_02_menu: GOOD: this-run open; desktop WASD/mouse copy; solid dim no ghost; ASCII -; no stick chrome | BAD: none
CRITIQUE qhd_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + difficulty; desktop help unclipped; no ghost | BAD: none
CRITIQUE qhd_04_playing: GOOD: this-run open; single blue border; no stick/DASH desktop; Score/hearts; Dash READY; entities inside; HUD ASCII | BAD: none
CRITIQUE qhd_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 1; ENTER/SPACE/ESC; no ghost | BAD: none

CRITIQUE 4k_01_boot: GOOD: this-run open; title + CTA settled 3840×2160 (UI sparse OK for CSS 4K); not blank | BAD: none
CRITIQUE 4k_02_menu: GOOD: this-run open; desktop WASD/mouse copy; solid dim no ghost; ASCII -; no stick chrome | BAD: none
CRITIQUE 4k_03_mode_select: GOOD: this-run open; SELECT MODE 4 modes + difficulty; help unclipped; no ghost | BAD: none
CRITIQUE 4k_04_playing: GOOD: this-run open; single blue border; no stick/DASH desktop; Score/hearts; Dash READY; entities inside; HUD ASCII | BAD: none
CRITIQUE 4k_05_game_over: GOOD: this-run open; GAME OVER unclipped; CLASSIC - NORMAL - Score 1; ENTER/SPACE/ESC; no ghost | BAD: none

CRITIQUE laptop_hd_01_boot: GOOD: recheck this-run open; title + CTA 1366×768; not blank | BAD: none
CRITIQUE laptop_hd_02_menu: GOOD: recheck this-run open; desktop WASD/mouse copy; solid dim no ghost; ASCII - | BAD: none
CRITIQUE laptop_hd_03_mode_select: GOOD: prior this-run open lines stand; modes+diff; desktop help | BAD: none
CRITIQUE laptop_hd_04_playing: GOOD: recheck this-run open; single border; no stick/DASH; Dash READY; entities inside; HUD ASCII | BAD: none
CRITIQUE laptop_hd_05_game_over: GOOD: prior this-run open lines stand; GO desktop hints | BAD: none

CRITIQUE 1080p_01_boot: GOOD: recheck this-run open; title + CTA 1920×1080; not blank | BAD: none
CRITIQUE 1080p_02_menu: GOOD: recheck this-run open; desktop WASD/mouse copy; solid dim no ghost; ASCII - | BAD: none
CRITIQUE 1080p_03_mode_select: GOOD: prior this-run open lines stand; modes+diff | BAD: none
CRITIQUE 1080p_04_playing: GOOD: recheck this-run open; single border; no stick/DASH; Dash READY; entities inside; HUD ASCII | BAD: none
CRITIQUE 1080p_05_game_over: GOOD: recheck this-run open; GAME OVER unclipped CLASSIC - NORMAL - Score 0; ENTER/SPACE/ESC; no ghost | BAD: none

**open_bads desktop residual matrix (this section):** none

## 2026-07-17 Fortnite stick ship (A6) — emulator recapture
Run: stick ship worker; run_start_unix 1784321756; criteria scripts/qa_success_criteria.json
CAPTURE path: e2e_emulator_matrix EMU_REQUIRE=1 CAPTURE_MATRIX=1 formats phone_android,phone_android_landscape,phone_portrait,phone_landscape
REVIEW: opened this-run PNGs with image tool (playing + midplay stills + menu/mode/GO/boot).
Scope: partial recapture for Fortnite stick visuals (full matrix not re-run; prior d886814 residual for other formats).
---
CRITIQUE phone_android_01_boot: GOOD: title RUSTY DASHER + CTA settled; portrait full-display emu; no Translate; qa_matrix URL | BAD: none
CRITIQUE phone_android_02_menu: GOOD: stick/DASH phone copy + swap strip; Best 3; no · tofu; portrait | BAD: none
CRITIQUE phone_android_03_mode_select: GOOD: SELECT MODE all 4 + NORMAL + green START + touch hints; START clear of help | BAD: none
CRITIQUE phone_android_04_playing: GOOD: Fortnite stick = translucent base + white ring + light knob; DASH red fill + white rim; floating HUD no solid deck; both fully outside blue playfield (V-PLAY-CONTROLS-OUTSIDE-FIELD); single border no nested frames; no side dim slabs; Score/hearts HUD; entities in field | BAD: none
CRITIQUE phone_android_05_game_over: GOOD: real GAME OVER unclipped SURVIVAL/NORMAL Score 0 + touch hints; force_go | BAD: none
CRITIQUE phone_android_landscape_01_boot: GOOD: title+CTA settled; landscape full-display emu | BAD: none (lab Chrome URL bar OK)
CRITIQUE phone_android_landscape_02_menu: GOOD: stick/DASH copy + swap; landscape panel fits | BAD: none
CRITIQUE phone_android_landscape_03_mode_select: GOOD: modes+diff+START readable landscape | BAD: none
CRITIQUE phone_android_landscape_04_playing: GOOD: Fortnite white ring+knob left grip; DASH+white rim right grip outside field; floating no solid PSP slabs; single border; Score/hearts | BAD: none
CRITIQUE phone_android_landscape_05_game_over: GOOD: real GAME OVER CLASSIC/NORMAL + touch hints landscape; force_go | BAD: none
CRITIQUE phone_portrait_01_boot: GOOD: title+CTA settled portrait | BAD: none
CRITIQUE phone_portrait_02_menu: GOOD: stick/DASH copy + swap strip; panel clean | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: modes+diff+START+hints; START clear | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: real GAME OVER SURVIVAL/NORMAL Score 0 + touch hints; force_go | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: title+CTA settled landscape | BAD: none
CRITIQUE phone_landscape_02_menu: GOOD: stick/DASH copy + swap; panel fits landscape | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: modes+diff+wide START landscape | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: real GAME OVER CLASSIC/NORMAL Score 0 + touch hints; force_go | BAD: none
# Desktop residual spot-check (prior ship PNG; no stick codepath):
CRITIQUE 1080p_04_playing: GOOD: desktop full field; no stick/DASH chrome (V-PLAY-DESKTOP-NO-STICK); Dash READY; single border | BAD: none [prior-ship artifact; stick change is touch-only]
CRITIQUE phone_portrait_04_playing: GOOD: hazards inside blue border; HUD mode|diff clear; stick+DASH outside field | BAD: none
CRITIQUE phone_landscape_04_playing: GOOD: hazards inside blue border; HUD mode|diff clear; stick+DASH outside field | BAD: none
CRITIQUE phone_rodin_04_playing: GOOD: hazards inside blue border; HUD mode|diff clear; stick+DASH outside field | BAD: none

## 2026-07-17 Phase A OOB/HUD product fix — scoped A7 (ship-consolidate)
Run: ship-consolidate; criteria scripts/qa_success_criteria.json v5; product world/ui/viewport hazard in-bounds + HUD z=50.
**A7 scope: PASS for recaptured product-fix cells only — NOT full matrix A7.**
Desktop baseline replay: INSPECT_VIDEOS (still match layout/HUD/feel; final_state GO vs playing = RNG; not feel-breaking).
---
CRITIQUE phone_portrait_04_playing: GOOD: this-run fix recapture; hazards in blue rect; HUD mode|diff z-clear; Game Boy stick+DASH outside field | BAD: none
CRITIQUE phone_landscape_04_playing: GOOD: this-run fix recapture; hazards in blue rect; HUD clear; PSP stick+DASH outside field | BAD: none
CRITIQUE phone_rodin_04_playing: GOOD: this-run fix recapture; hazards in blue rect; HUD clear; stick+DASH outside field | BAD: none
# Residual (pre-fix art / not re-reviewed this ship for OOB/HUD):
# phone_android_landscape_04_playing.review.json still has V-PLAY-HAZARD-NOT-ON-HUD issues from pre-fix open
# Other touch formats (iphone, samsung, large, rodin_landscape, rodin_chrome midplay, tablet, etc.) keep prior A6/Fortnite art

## Fix-ship review 2026-07-17T22:34:18Z (run_start_unix=1784327295) — scoped phone_portrait + phone_landscape
CRITIQUE phone_portrait_01_boot: GOOD: boot overlay visible | BAD: none
CRITIQUE phone_portrait_02_menu: GOOD: full blue panel L/R margins, swap band inside, no lateral clip | BAD: none
CRITIQUE phone_portrait_03_mode_select: GOOD: full panel laterals, START clear, help inside bottom | BAD: none
CRITIQUE phone_portrait_04_playing: GOOD: stick+DASH outside field; no Dash READY/X.Xs on chrome | BAD: none
CRITIQUE phone_portrait_05_game_over: GOOD: GO panel full border with side margins | BAD: none
CRITIQUE phone_landscape_01_boot: GOOD: boot present | BAD: none
CRITIQUE phone_landscape_02_menu: GOOD: panel centered full border no lateral clip | BAD: none
CRITIQUE phone_landscape_03_mode_select: GOOD: full border START clear | BAD: none
CRITIQUE phone_landscape_04_playing: GOOD: grips outside field no Dash status on chrome | BAD: none
CRITIQUE phone_landscape_05_game_over: GOOD: GO panel in canvas | BAD: none
