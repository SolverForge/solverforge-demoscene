// Thin C shim over the vendored glyphfx engine: parse a glyphfx-style argv,
// build one effect on a virtual 60fps clock, and step it frame by frame.
// Same argv + text -> byte-identical frame stream (glyphfx parity guarantee).
#include <stdlib.h>
#include <string.h>

#include "cli.h"
#include "effects/registry.h"
#include "engine/ctx.h"
#include "engine/render_log.h"
#include "utils/clock.h"
#include "utils/rng.h"

// glyphfx keeps one engine per process: the render log and its target
// terminal are globals. Detach them between clips so a new clip never logs
// moves against the previous clip's freed terminal.
static void detach_render_log(void) {
    render_log_bind(NULL);
    g_rlog.len = 0;
    g_rlog.on = false;
    g_rlog.overflow = false;
}

typedef struct {
    CliConfig cfg;
    EngineCtx ctx;
    Effect *effect;
} GfxClip;

GfxClip *gfx_open(int argc, char **argv, const char *text) {
    GfxClip *clip = calloc(1, sizeof(GfxClip));
    if (!clip) {
        return NULL;
    }
    if (cli_parse(argc, argv, &clip->cfg) != 0 || !clip->cfg.effect_entry) {
        free(clip);
        return NULL;
    }
    const EffectEntry *entry = (const EffectEntry *)clip->cfg.effect_entry;
    clip->effect = entry->make(clip->cfg.effect_config);
    if (!clip->effect) {
        free(clip);
        return NULL;
    }
    detach_render_log();
    Rng rng = rng_seeded(clip->cfg.has_seed ? clip->cfg.seed : 1);
    Clock clock = clock_virtual_with_frame_rate(clip->cfg.tc.frame_rate);
    PreprocessError err;
    memset(&err, 0, sizeof(err));
    if (engine_ctx_init(&clip->ctx, text, &clip->cfg.tc, rng, clock, &err) != 0) {
        clip->effect->ops->destroy(clip->effect);
        free(clip);
        return NULL;
    }
    if (clip->effect->ops->build(clip->effect, &clip->ctx) != 0) {
        clip->effect->ops->destroy(clip->effect);
        engine_ctx_free(&clip->ctx);
        free(clip);
        return NULL;
    }
    return clip;
}

// Returns the next frame (owned by the engine, valid until the next call), or
// NULL when the effect has finished.
const char *gfx_next(GfxClip *clip) {
    return clip->effect->ops->next_frame(clip->effect, &clip->ctx);
}

void gfx_close(GfxClip *clip) {
    if (!clip) {
        return;
    }
    const EffectEntry *entry = (const EffectEntry *)clip->cfg.effect_entry;
    clip->effect->ops->destroy(clip->effect);
    engine_ctx_free(&clip->ctx);
    detach_render_log();
    entry->free_config(clip->cfg.effect_config);
    free(clip->cfg.effect_config);
    free(clip);
}
