<script>
    import CodeMirror from "svelte-codemirror-editor";
    import { basicSetup } from "codemirror";
    import { oneDark } from "@codemirror/theme-one-dark";
    import { wgsl } from "@iizukak/codemirror-lang-wgsl";
    import { vim } from "@replit/codemirror-vim";
    import { getDimensions } from "$lib/wgputoy.js";
    import {
        manual_reload,
        needs_initial_reset,
        is_safe_context,
        wgpu_renderer,
        parse_error,
        resize,
        code,
    } from "$lib/canvas/stores.js";
    import { compress, decompress } from "$lib/util.js";

    export let bindID = "wgputoy-canvas";

    let default_shader = `G0sDABwFbtNzOZJEv8ZQJDrMt2bR/j7ViXsPYArkFxPExQxycT6i1UW7Unjv4HL6sxJ84pS61Zo0t45F99anGwNcgHnIaebWHkEGPdOx3eZwEapRB5POBGUkpUSXLhh7pFpjVnqHMHaZCiUwGtNOjOYK9sfR1J5X0DLfPY+u0e/MVwDI8thx1SJFCHLDypWRDtbuBoQEoIYQ2Jv7pfs2ckyDsZyZSw8V1AzXlklo0rvi2L8PU3wi638AmHoQrfRCFG6QWf39FEZ9D9/M94Hm3h8LKqPv6IM0YoEG2/LfIZYQhvZX+s/6SzXkTMoIbTin9e51DeAOw0M6QZ4fTwUizioj0mjL8FRPb0zXrsUJHEaHB4o/oCK4vz8QwnBkn0gtA+OYNOCSZG0ZJMJCzJohJB36atBn+Uq+NpO6tTph+8Dj/pBe7xd4h/m1tyIJkYSVmCzp9kSol4Zq2yrxHYEkfLL0DOa9bgJS93Q86SoniIgdAW3nubuQE2t5DMSRLSSeSL30xCZlQdWU3wJ8w9hpjlWgxiIM`;
    var params = window.location.hash.substring(1);

    if (params.length < 0 || params == null || params == "") {
        params = default_shader;
    }

    let param_promise = decompress(params);

    let change_timer;

    let init = performance.now();
    let last = performance.now();
    let timer = 0;

    let value = "";
    let wgputoy = null;

    param_promise.then((promise) => {
        $code = promise;
        value = promise;
        handle_change();
    });

    function debounce(func, delay) {
        return function () {
            clearTimeout(change_timer);
            change_timer = setTimeout(func, delay);
        };
    }

    const handle_change = debounce(async () => {
        $code = value;
        if (is_safe_context(wgputoy)) {
            reload();
        }

        window.location.hash = await compress($code);

        // Reset the resize flag after a short delay
        setTimeout(() => ($resize = false), 100);
    }, 250);

    export const reload = () => {
        update_uniforms().then(() => {
            if (is_safe_context(wgputoy)) {
                console.log("reloading");
                wgputoy.preprocess($code).then((s) => {
                    if (s) {
                        wgputoy.compile(s);
                        // setPrelude(wgputoy.prelude()); // TODO: this is not working
                        wgputoy.render();
                    }
                });
                $manual_reload = false;
            }
        });
    };

    export const update_uniforms = async () => {
        if (is_safe_context(wgputoy)) {
            const names = [];
            const values = [];
            // [...sliderRefMap().keys()].map((uuid) => {
            //     names.push(sliderRefMap().get(uuid).getUniform());
            //     values.push(sliderRefMap().get(uuid).getVal());
            // }, this);
            if (names.length > 0) {
                await wgputoy.set_custom_floats(
                    names,
                    Float32Array.from(values),
                );
            }
            // setSliderUpdateSignal(false);
        }
    };

    export const awaitable_reload = async () => {
        await update_uniforms();
        if (is_safe_context(wgputoy)) {
            const s = await wgputoy.preprocess($code);
            if (s) {
                wgputoy.compile(s);
                //     // setPrelude(wgputoy.prelude()); // TODO: this is not working
                wgputoy.render();
            }
            return true;
        } else {
            return false;
        }
    };

    const update_resolution = () => {
        if (is_safe_context(wgputoy)) {
            let dimensions = getDimensions(bindID);
            const newScale = 1;
            wgputoy.resize(dimensions.x, dimensions.y, newScale);
            reload();
        }
    };

    export const reset = () => {
        if (is_safe_context(wgputoy)) {
            wgputoy.reset();
            reload();
        }
    };

    export const handle_success = (_entryPoints) => {
        // setEntryPoints(entryPoints);
        $parse_error = {
            success: true,
        };
    };

    export const handle_error = (summary, row, col) => {
        $parse_error = {
            summary: summary,
            position: { row: Number(row), col: Number(col) },
            success: false,
        };
    };

    export const live_reload = (deltaTime, timer) => {
        wgputoy.set_time_delta(deltaTime);
        wgputoy.set_time_elapsed(timer);
        wgputoy.render();
    };

    const loop = async (now) => {
        if ($needs_initial_reset) {
            console.log("needs initial reset");
            const ready = await awaitable_reload();
            if (ready && $parse_error.success) {
                console.log("got here");
                reset();
                $needs_initial_reset = false;
            }
        } else if ($manual_reload) {
            console.log("manual reload");
            reload();
        }

        let time = (now - init) / 1000;
        let delta = (now - last) / 1000;
        timer += delta;

        live_reload(delta, time);
        last = now;
        requestAnimationFrame(loop);
    };

    $: if ($wgpu_renderer && $resize) {
        update_resolution();
        $resize = false;
    }

    $: if ($wgpu_renderer && param_promise) {
        wgputoy = $wgpu_renderer;
        update_resolution();

        wgputoy.on_success(handle_success);
        if (window) window["wgsl_error_handler"] = handle_error;

        requestAnimationFrame(loop);
    }
</script>

{#if param_promise}
    <CodeMirror
        class="text_edit"
        bind:value
        on:change={handle_change}
        lang={wgsl()}
        theme={oneDark}
        extensions={[vim(), basicSetup]}
    />
{/if}

<style>
</style>
