# Desktop baseline (protect desktop feel)

Recorded: **2026-07-17T22:00:04.607Z**  
Format: **1080p** 1920×1080  
Video: `baseline_1080p_keyboard.webm`  
Inputs: `inputs.json` (exact timed key sequence)  
Metrics: `metrics_record.json`

## After mobile-only changes

```bash
./scripts/web-serve-dist.sh
node scripts/desktop_baseline.mjs replay
node scripts/desktop_baseline.mjs compare
# open baseline_*.webm vs replay_*.webm
```

If desktop motion/dash/layout changed, **revert non-handheld-gated code**.
