//! PyO3 methods expanded before its single impl is parsed.

macro_rules! plot_api {
    (
        text { $($text:ident : $text_cap:ident =>
            $clear_style:ident, $clear_layout:ident;)* }
        clear { $($clear:ident : $clear_cap:ident;)* }
        layout { $($layout:ident : $layout_cap:ident;)* }
        format { $($format:ident : $format_cap:ident => $slot:ident;)* }
        generation { $($generation:ident : $generation_cap:ident => $tick_slot:ident;)* }
        flag { $($flag:ident : $flag_cap:ident;)* }
        manual { $($manual:item)* }
    ) => {
        #[pymethods]
        impl PyPlotOwner {
            $(
                fn $text(
                    slf: &Bound<'_, Self>,
                    text: &Bound<'_, PyAny>,
                    style: &Bound<'_, PyAny>,
                    layout: &Bound<'_, PyAny>,
                    unset: &Bound<'_, PyAny>,
                ) -> PyResult<()> {
                    let intent = values::text_intent(text, style, layout, unset)?;
                    Self::update(slf, |native| {
                        map_capability!($text_cap, native, plot => {
                            let mut plot = plot.$text(intent.native);
                            if intent.clear_style {
                                plot = plot.$clear_style();
                            }
                            if intent.clear_layout {
                                plot = plot.$clear_layout();
                            }
                            plot
                        })
                    })
                }
            )*
            $(
                fn $clear(slf: &Bound<'_, Self>) -> PyResult<()> {
                    Self::update(slf, |native| {
                        map_capability!($clear_cap, native, plot => plot.$clear())
                    })
                }
            )*
            $(
                fn $layout(
                    slf: &Bound<'_, Self>,
                    layout: PyRef<'_, PyTextLayout>,
                ) -> PyResult<()> {
                    let native_layout = layout.native.clone();
                    drop(layout);
                    Self::update(slf, |native| {
                        map_capability!($layout_cap, native, plot => plot.$layout(native_layout))
                    })
                }
            )*
            $(
                fn $format(
                    slf: &Bound<'_, Self>,
                    format: PyRef<'_, PyNumericFormat>,
                ) -> PyResult<()> {
                    let (native_format, roots) = format.snapshot(slf.py())?;
                    drop(format);
                    Self::update_callback(slf, CallbackSlot::$slot, roots, |native| {
                        map_capability!($format_cap, native, plot => plot.$format(native_format))
                    })
                }
            )*
            $(
                fn $generation(
                    slf: &Bound<'_, Self>,
                    generation: PyRef<'_, PyTickGeneration>,
                ) -> PyResult<()> {
                    let (native_generation, roots) = generation.snapshot(slf.py())?;
                    drop(generation);
                    Self::update_callback(slf, CallbackSlot::$tick_slot, roots, |native| {
                        map_capability!($generation_cap, native, plot =>
                            plot.$generation(native_generation))
                    })
                }
            )*
            $(
                fn $flag(slf: &Bound<'_, Self>, visible: bool) -> PyResult<()> {
                    Self::update(slf, |native| {
                        map_capability!($flag_cap, native, plot => plot.$flag(visible))
                    })
                }
            )*
            $($manual)*
        }
    };
}
