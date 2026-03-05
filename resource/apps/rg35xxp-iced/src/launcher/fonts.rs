pub fn load() {
    #[cfg(feature = "material-design-icons")]
    {
        use std::sync::Once;
        static INSTALLED: Once = Once::new();
        INSTALLED.call_once(|| {
            let mut write = iced_graphics::text::font_system().write().unwrap();
            write.load_font(
        include_bytes!(
            "../../assets/material-design-icons/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf"
        )
            .into(),
    );
            write.load_font(
            include_bytes!(
                "../../assets/material-design-icons/MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf"
            )
            .into(),
        );
            write.load_font(
            include_bytes!(
                "../../assets/material-design-icons/MaterialSymbolsRounded[FILL,GRAD,opsz,wght].ttf"
            )
            .into(),
        );
        });
    }
}
