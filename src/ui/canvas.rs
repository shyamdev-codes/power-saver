use std::ptr::null_mut;

use crate::app::{AppState, SettingsActiveField};
use crate::ui::theme::*;
use crate::win32::*;

pub unsafe fn render_ui(hdc: HDC, app: &AppState) {
    let width = 960;
    let height = 520;

    unsafe {
        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bmp = CreateCompatibleBitmap(hdc, width, height);
        let old_bmp = SelectObject(mem_dc, mem_bmp);

        SetBkMode(mem_dc, TRANSPARENT);

        // 1. Deep Space Slate-Navy Background (#0B131F)
        let bg_brush = CreateSolidBrush(COLOR_BG);
        let rc_all = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        FillRect(mem_dc, &rc_all, bg_brush);
        DeleteObject(bg_brush);

        // 2. High-Legibility ClearType Typography Helper
        let make_font = |h: i32, w: i32| -> HFONT {
            CreateFontW(
                h, 0, 0, 0, w, 0, 0, 0,
                1, // DEFAULT_CHARSET
                0, // OUT_DEFAULT_PRECIS
                0, // CLIP_DEFAULT_PRECIS
                5, // CLEARTYPE_QUALITY
                0, // DEFAULT_PITCH
                to_wide("Segoe UI").as_ptr(),
            )
        };

        let font_title = make_font(24, 700);          // Large bold header
        let font_sub = make_font(13, 400);            // Clean hardware line
        let font_pill = make_font(13, 700);           // Autonomous state badge
        let font_sec_head = make_font(14, 700);       // Section titles
        let font_metric_num = make_font(24, 700);     // 24px bold live metrics
        let font_metric_lbl = make_font(11, 700);     // Metric labels
        let font_card_val = make_font(21, 700);       // Equation card values
        let font_card_lbl = make_font(11, 700);       // Equation card labels
        let font_card_sub = make_font(10, 500);       // Equation card sub-labels
        let font_operator = make_font(24, 700);       // Equation − and =
        let font_active_runtime = make_font(15, 600); // 15px bold runtime indicator
        let font_insight_head = make_font(12, 700);   // Insight box title
        let font_insight_body = make_font(14, 500);   // 14px crystal clear insight text
        let font_legend = make_font(14, 600);         // Chart legend
        let font_axis = make_font(13, 500);           // Axis labels
        let font_status = make_font(13, 600);         // Bottom status bar
        let font_modal_title = make_font(18, 700);    // Settings modal header
        let font_input_text = make_font(18, 600);     // Modal input field content
        let font_input_lbl = make_font(11, 700);      // Modal input labels
        let font_input_hint = make_font(11, 400);     // Modal input field hints
        let font_modal_btn = make_font(13, 700);      // Modal action buttons

        // =========================================================================
        // HEADER BAR (Y: 12 .. 58)
        // =========================================================================
        let old_font = SelectObject(mem_dc, font_title);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_title = RECT {
            left: 20,
            top: 12,
            right: 500,
            bottom: 38,
        };
        let title_str = to_wide("ADAPTIVE POWER GOVERNOR");
        DrawTextW(mem_dc, title_str.as_ptr(), -1, &mut rc_title, DT_LEFT | DT_SINGLELINE);

        SelectObject(mem_dc, font_sub);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_sub = RECT {
            left: 22,
            top: 38,
            right: 500,
            bottom: 58,
        };
        let sub_str = to_wide(&format!("{} · Win32 Kernel Driver", app.cpu_name));
        DrawTextW(mem_dc, sub_str.as_ptr(), -1, &mut rc_sub, DT_LEFT | DT_SINGLELINE);

        // Header Action: [ ⚙ Settings ] Button (X: 510 .. 636, Y: 14 .. 48)
        let settings_btn_brush = CreateSolidBrush(COLOR_TILE_BG);
        let settings_btn_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_BORDER);
        SelectObject(mem_dc, settings_btn_brush);
        SelectObject(mem_dc, settings_btn_pen);
        RoundRect(mem_dc, 510, 14, 636, 48, 12, 12);
        DeleteObject(settings_btn_brush);
        DeleteObject(settings_btn_pen);

        SelectObject(mem_dc, font_pill);
        SetTextColor(mem_dc, COLOR_TEXT_PRIMARY);
        let mut rc_sbtn = RECT { left: 510, top: 14, right: 636, bottom: 48 };
        let sbtn_str = to_wide("⚙ Settings");
        DrawTextW(mem_dc, sbtn_str.as_ptr(), -1, &mut rc_sbtn, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        // Top Right: Autonomous State Badge Pill (X: 650 .. 940, Y: 14 .. 48)
        let active_limit = app.current_limit();
        let (badge_text, badge_bg, badge_fg, badge_border) = match active_limit {
            100 => ("● AUTONOMOUS: BOOST 100%", COLOR_CYAN_BG, COLOR_CYAN, rgb(2, 132, 199)),
            80 => ("● AUTONOMOUS: BALANCED 80%", COLOR_AMBER_BG, COLOR_AMBER, rgb(217, 119, 6)),
            _ => ("● AUTONOMOUS: DEEP ECO 70%", COLOR_EMERALD_BG, COLOR_EMERALD, rgb(5, 150, 105)),
        };

        let badge_brush = CreateSolidBrush(badge_bg);
        let badge_pen = CreatePen(PS_SOLID, 1, badge_border);
        let old_brush = SelectObject(mem_dc, badge_brush);
        let old_pen = SelectObject(mem_dc, badge_pen);
        RoundRect(mem_dc, 650, 14, 940, 48, 14, 14);
        SelectObject(mem_dc, old_brush);
        SelectObject(mem_dc, old_pen);
        DeleteObject(badge_brush);
        DeleteObject(badge_pen);

        SelectObject(mem_dc, font_pill);
        SetTextColor(mem_dc, badge_fg);
        let mut rc_badge_txt = RECT {
            left: 650,
            top: 14,
            right: 940,
            bottom: 48,
        };
        let w_badge = to_wide(badge_text);
        DrawTextW(mem_dc, w_badge.as_ptr(), -1, &mut rc_badge_txt, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        // =========================================================================
        // TWO-COLUMN ELEVATED CARDS (Y: 64 .. 458, Height = 394 px)
        // =========================================================================
        let card_brush = CreateSolidBrush(COLOR_CARD_BG);
        let card_border_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_BORDER);

        // --- LEFT CARD (X: 20 .. 472, Width = 452) ---
        SelectObject(mem_dc, card_brush);
        SelectObject(mem_dc, card_border_pen);
        RoundRect(mem_dc, 20, 64, 472, 458, 12, 12);

        // Left Card Header: SYSTEM TELEMETRY
        SelectObject(mem_dc, font_sec_head);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_ltitle = RECT { left: 36, top: 76, right: 260, bottom: 94 };
        let ltitle_str = to_wide("SYSTEM TELEMETRY");
        DrawTextW(mem_dc, ltitle_str.as_ptr(), -1, &mut rc_ltitle, DT_LEFT | DT_SINGLELINE);

        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_live_tag = RECT { left: 270, top: 76, right: 456, bottom: 94 };
        let live_tag_str = to_wide("● REAL-TIME");
        DrawTextW(mem_dc, live_tag_str.as_ptr(), -1, &mut rc_live_tag, DT_RIGHT | DT_SINGLELINE);

        // 3 Large Metric Sub-Tiles (Y: 98 .. 166, Height: 68)
        let tile_brush = CreateSolidBrush(COLOR_TILE_BG);
        let tile_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_BORDER);

        let draw_tile = |dc: HDC, x1: i32, y1: i32, x2: i32, y2: i32, val: &str, label: &str, val_col: u32| {
            SelectObject(dc, tile_brush);
            SelectObject(dc, tile_pen);
            RoundRect(dc, x1, y1, x2, y2, 8, 8);

            SelectObject(dc, font_metric_num);
            SetTextColor(dc, val_col);
            let mut rc_v = RECT { left: x1, top: y1 + 8, right: x2, bottom: y1 + 38 };
            let w_val = to_wide(val);
            DrawTextW(dc, w_val.as_ptr(), -1, &mut rc_v, DT_CENTER | DT_SINGLELINE);

            SelectObject(dc, font_metric_lbl);
            SetTextColor(dc, COLOR_TEXT_LABEL);
            let mut rc_l = RECT { left: x1, top: y1 + 40, right: x2, bottom: y2 - 6 };
            let w_lbl = to_wide(label);
            DrawTextW(dc, w_lbl.as_ptr(), -1, &mut rc_l, DT_CENTER | DT_SINGLELINE);
        };

        // Tile 1: HARDWARE CLOCK (36 .. 168)
        draw_tile(mem_dc, 36, 98, 168, 166, &format!("{} MHz", app.current_mhz), "HARDWARE CLOCK", COLOR_TEXT_TITLE);

        // Tile 2: CPU LOAD (180 .. 312)
        let load_col = if app.current_load >= 70.0 {
            COLOR_RED
        } else if app.current_load >= 35.0 {
            COLOR_AMBER
        } else {
            COLOR_CYAN
        };
        draw_tile(mem_dc, 180, 98, 312, 166, &format!("{:.0}%", app.current_load), "CPU LOAD", load_col);

        // Tile 3: ACTIVE CAP PROFILE (324 .. 456)
        let cap_str = format!("{}% CAP", app.current_limit());
        draw_tile(mem_dc, 324, 98, 456, 166, &cap_str, "GOVERNOR STATE", badge_fg);

        DeleteObject(tile_brush);
        DeleteObject(tile_pen);

        // Subtle Section Divider Line (Y = 176)
        let div_pen = CreatePen(PS_SOLID, 1, COLOR_DIVIDER);
        let op = SelectObject(mem_dc, div_pen);
        MoveToEx(mem_dc, 36, 176, null_mut());
        LineTo(mem_dc, 456, 176);
        SelectObject(mem_dc, op);
        DeleteObject(div_pen);

        // Savings Accumulation Math using User-Configured Tariff
        let cum_total_secs = app.today_base_active_secs + app.session_seconds;
        let cum_eco_secs = app.today_base_eco_secs + app.eco_seconds;
        let hours = cum_total_secs / 3600;
        let mins = (cum_total_secs % 3600) / 60;
        let time_str = if hours > 0 { format!("{}h {}m", hours, mins) } else { format!("{}m", mins) };
        let eco_pct = if cum_total_secs > 0 { ((cum_eco_secs as f64 / cum_total_secs as f64) * 100.0).round() as u64 } else { 100 };
        let wh_saved = ((app.today_base_wh + (app.total_watt_seconds_saved / 3600.0)) * 10.0).round() / 10.0;
        let today_rupees = (((wh_saved / 1000.0) * app.settings.tariff_per_kwh) * 100.0).round() / 100.0;
        let delta_watts = (app.unthrottled_watts - app.live_watts).max(0.0);

        // Section Title: ENERGY SAVINGS EQUATION
        SelectObject(mem_dc, font_sec_head);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_imp_t = RECT { left: 36, top: 184, right: 250, bottom: 202 };
        let imp_t_str = to_wide("ENERGY SAVINGS EQUATION");
        DrawTextW(mem_dc, imp_t_str.as_ptr(), -1, &mut rc_imp_t, DT_LEFT | DT_SINGLELINE);

        SelectObject(mem_dc, font_sub);
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_today_sum = RECT { left: 240, top: 184, right: 456, bottom: 202 };
        let today_sum_str = to_wide(&format!("{} {:.2} today (at {} {:.2}/unit ✎)", app.settings.currency_symbol, today_rupees, app.settings.currency_symbol, app.settings.tariff_per_kwh));
        DrawTextW(mem_dc, today_sum_str.as_ptr(), -1, &mut rc_today_sum, DT_RIGHT | DT_SINGLELINE);

        // =========================================================================
        // 3 PROPER VISUAL CARDS ROW (Y: 208 .. 280, Height = 72 px)
        // [ Without Governor ]  -  [ Actual Draw ]  =  [ Total Saved ]
        // =========================================================================

        // --- Card 1: WITHOUT GOVERNOR (36 .. 156, Width = 120) ---
        let c1_brush = CreateSolidBrush(COLOR_CARD_AMBER_BG);
        let c1_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_AMBER_BORDER);
        SelectObject(mem_dc, c1_brush);
        SelectObject(mem_dc, c1_pen);
        RoundRect(mem_dc, 36, 208, 156, 280, 8, 8);
        DeleteObject(c1_brush);
        DeleteObject(c1_pen);

        SelectObject(mem_dc, font_card_val);
        SetTextColor(mem_dc, COLOR_AMBER);
        let mut rc_c1_v = RECT { left: 36, top: 213, right: 156, bottom: 239 };
        let w_c1_v = to_wide(&format!("{:.1} W", app.unthrottled_watts));
        DrawTextW(mem_dc, w_c1_v.as_ptr(), -1, &mut rc_c1_v, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_lbl);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_c1_l = RECT { left: 36, top: 241, right: 156, bottom: 257 };
        let w_c1_l = to_wide("WITHOUT GOVERNOR");
        DrawTextW(mem_dc, w_c1_l.as_ptr(), -1, &mut rc_c1_l, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_sub);
        SetTextColor(mem_dc, COLOR_AMBER);
        let mut rc_c1_s = RECT { left: 36, top: 258, right: 156, bottom: 274 };
        let w_c1_s = to_wide("100% Baseline");
        DrawTextW(mem_dc, w_c1_s.as_ptr(), -1, &mut rc_c1_s, DT_CENTER | DT_SINGLELINE);

        // --- Minus Connector: "−" (156 .. 180) ---
        SelectObject(mem_dc, font_operator);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_op1 = RECT { left: 156, top: 216, right: 180, bottom: 268 };
        let w_op1 = to_wide("−");
        DrawTextW(mem_dc, w_op1.as_ptr(), -1, &mut rc_op1, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        // --- Card 2: ACTUAL DRAW (180 .. 300, Width = 120) ---
        let c2_brush = CreateSolidBrush(COLOR_CARD_EMERALD_BG);
        let c2_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_EMERALD_BORDER);
        SelectObject(mem_dc, c2_brush);
        SelectObject(mem_dc, c2_pen);
        RoundRect(mem_dc, 180, 208, 300, 280, 8, 8);
        DeleteObject(c2_brush);
        DeleteObject(c2_pen);

        SelectObject(mem_dc, font_card_val);
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_c2_v = RECT { left: 180, top: 213, right: 300, bottom: 239 };
        let w_c2_v = to_wide(&format!("{:.1} W", app.live_watts));
        DrawTextW(mem_dc, w_c2_v.as_ptr(), -1, &mut rc_c2_v, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_lbl);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_c2_l = RECT { left: 180, top: 241, right: 300, bottom: 257 };
        let w_c2_l = to_wide("ACTUAL DRAW");
        DrawTextW(mem_dc, w_c2_l.as_ptr(), -1, &mut rc_c2_l, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_sub);
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_c2_s = RECT { left: 180, top: 258, right: 300, bottom: 274 };
        let w_c2_s = to_wide("Governed Cap");
        DrawTextW(mem_dc, w_c2_s.as_ptr(), -1, &mut rc_c2_s, DT_CENTER | DT_SINGLELINE);

        // --- Equals Connector: "=" (300 .. 324) ---
        SelectObject(mem_dc, font_operator);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_op2 = RECT { left: 300, top: 216, right: 324, bottom: 268 };
        let w_op2 = to_wide("=");
        DrawTextW(mem_dc, w_op2.as_ptr(), -1, &mut rc_op2, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

        // --- Card 3: TOTAL SAVED (324 .. 444, Width = 120) ---
        let c3_brush = CreateSolidBrush(COLOR_CARD_CYAN_BG);
        let c3_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_CYAN_BORDER);
        SelectObject(mem_dc, c3_brush);
        SelectObject(mem_dc, c3_pen);
        RoundRect(mem_dc, 324, 208, 444, 280, 8, 8);
        DeleteObject(c3_brush);
        DeleteObject(c3_pen);

        SelectObject(mem_dc, font_card_val);
        SetTextColor(mem_dc, COLOR_CYAN);
        let mut rc_c3_v = RECT { left: 324, top: 213, right: 444, bottom: 239 };
        let w_c3_v = to_wide(&format!("+{:.1} W", delta_watts));
        DrawTextW(mem_dc, w_c3_v.as_ptr(), -1, &mut rc_c3_v, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_lbl);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_c3_l = RECT { left: 324, top: 241, right: 444, bottom: 257 };
        let w_c3_l = to_wide("TOTAL SAVED");
        DrawTextW(mem_dc, w_c3_l.as_ptr(), -1, &mut rc_c3_l, DT_CENTER | DT_SINGLELINE);

        SelectObject(mem_dc, font_card_sub);
        SetTextColor(mem_dc, COLOR_CYAN);
        let mut rc_c3_s = RECT { left: 324, top: 258, right: 444, bottom: 274 };
        let w_c3_s = to_wide("Instant Saved");
        DrawTextW(mem_dc, w_c3_s.as_ptr(), -1, &mut rc_c3_s, DT_CENTER | DT_SINGLELINE);

        // Active runtime Line (15px Bold White/Slate - Crisp & Clear)
        SelectObject(mem_dc, font_active_runtime);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_sess_info = RECT { left: 36, top: 290, right: 456, bottom: 312 };
        let sess_info_str = to_wide(&format!("Active runtime: {}   ·   Eco efficiency: {}%", time_str, eco_pct));
        DrawTextW(mem_dc, sess_info_str.as_ptr(), -1, &mut rc_sess_info, DT_LEFT | DT_SINGLELINE);

        // Dedicated Elevated Governor Insight Card (Y: 320 .. 444, Height = 124 px)
        let insight_box_brush = CreateSolidBrush(COLOR_INSIGHT_BG);
        let insight_box_pen = CreatePen(PS_SOLID, 1, COLOR_INSIGHT_BORDER);
        SelectObject(mem_dc, insight_box_brush);
        SelectObject(mem_dc, insight_box_pen);
        RoundRect(mem_dc, 36, 320, 456, 444, 8, 8);
        DeleteObject(insight_box_brush);
        DeleteObject(insight_box_pen);

        SelectObject(mem_dc, font_insight_head);
        SetTextColor(mem_dc, COLOR_ACCENT_BLUE);
        let mut rc_ins_h = RECT { left: 52, top: 328, right: 440, bottom: 346 };
        let ins_h_str = to_wide("GOVERNOR INSIGHT");
        DrawTextW(mem_dc, ins_h_str.as_ptr(), -1, &mut rc_ins_h, DT_LEFT | DT_SINGLELINE);

        SelectObject(mem_dc, font_insight_body);
        SetTextColor(mem_dc, COLOR_TEXT_TITLE);
        let mut rc_insight = RECT { left: 52, top: 348, right: 440, bottom: 436 };
        let insight_str = to_wide(&app.status_subtext);
        DrawTextW(mem_dc, insight_str.as_ptr(), -1, &mut rc_insight, DT_LEFT | DT_WORDBREAK);

        // --- RIGHT CARD: POWER CONSUMPTION TRACE (60s) (X: 488 .. 940, Width = 452) ---
        SelectObject(mem_dc, card_brush);
        SelectObject(mem_dc, card_border_pen);
        RoundRect(mem_dc, 488, 64, 940, 458, 12, 12);
        DeleteObject(card_brush);
        DeleteObject(card_border_pen);

        // Right Card Title
        SelectObject(mem_dc, font_sec_head);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_rtitle = RECT { left: 504, top: 76, right: 780, bottom: 94 };
        let rtitle_str = to_wide("POWER CONSUMPTION TRACE (60s)");
        DrawTextW(mem_dc, rtitle_str.as_ptr(), -1, &mut rc_rtitle, DT_LEFT | DT_SINGLELINE);

        // Current wattage tag pill
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_rtag = RECT { left: 780, top: 76, right: 924, bottom: 94 };
        let rtag_str = to_wide(&format!("[{:.1} W]", app.live_watts));
        DrawTextW(mem_dc, rtag_str.as_ptr(), -1, &mut rc_rtag, DT_RIGHT | DT_SINGLELINE);

        // Chart Canvas Dimensions (370 × 270 px)
        let cx = 554;
        let cy = 104;
        let cw = 370;
        let ch = 270;

        // Rounded Canvas Surface (#0D1624)
        let canvas_brush = CreateSolidBrush(COLOR_CANVAS_BG);
        let canvas_pen = CreatePen(PS_SOLID, 1, COLOR_GRID_PEN);
        SelectObject(mem_dc, canvas_brush);
        SelectObject(mem_dc, canvas_pen);
        RoundRect(mem_dc, cx, cy, cx + cw, cy + ch, 8, 8);
        DeleteObject(canvas_brush);
        DeleteObject(canvas_pen);

        // Grid Lines at 45W, 30W, 15W
        let grid_pen = CreatePen(PS_DASH, 1, COLOR_GRID_PEN);
        let op = SelectObject(mem_dc, grid_pen);
        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        SelectObject(mem_dc, font_axis);

        for w_val in [45.0, 30.0, 15.0] {
            let gy = cy + (ch as f64 * (1.0 - (w_val / 65.0))) as i32;
            MoveToEx(mem_dc, cx, gy, null_mut());
            LineTo(mem_dc, cx + cw, gy);

            // Left Y-axis label (13px ClearType)
            let mut rc_y = RECT { left: 500, top: gy - 10, right: cx - 8, bottom: gy + 10 };
            let y_str = to_wide(&format!("{:.0}W", w_val));
            DrawTextW(mem_dc, y_str.as_ptr(), -1, &mut rc_y, DT_RIGHT | DT_SINGLELINE);
        }
        SelectObject(mem_dc, op);
        DeleteObject(grid_pen);

        // Filled Polygon Shading under Actual Live Watts
        let step_x = cw as f64 / 59.0;
        let mut poly_points: Vec<POINT> = Vec::with_capacity(62);
        poly_points.push(POINT { x: cx, y: cy + ch - 1 });
        for i in 0..app.live_watts_history.len() {
            let x = cx + (i as f64 * step_x) as i32;
            let val = app.live_watts_history[i].min(65.0);
            let y = cy + (ch as f64 * (1.0 - (val / 65.0))) as i32;
            poly_points.push(POINT { x, y });
        }
        poly_points.push(POINT { x: cx + cw, y: cy + ch - 1 });

        let fill_brush = CreateSolidBrush(COLOR_EMERALD_FILL);
        let null_pen = CreatePen(PS_SOLID, 1, COLOR_EMERALD_FILL);
        let op_brush = SelectObject(mem_dc, fill_brush);
        let op_pen = SelectObject(mem_dc, null_pen);
        Polygon(mem_dc, poly_points.as_ptr(), poly_points.len() as i32);
        SelectObject(mem_dc, op_brush);
        SelectObject(mem_dc, op_pen);
        DeleteObject(fill_brush);
        DeleteObject(null_pen);

        // Plot Without Governor Baseline (Amber Dashed)
        let un_pen = CreatePen(PS_DASH, 1, COLOR_AMBER);
        let op = SelectObject(mem_dc, un_pen);
        for i in 0..app.unthrottled_history.len() {
            let x = cx + (i as f64 * step_x) as i32;
            let val = app.unthrottled_history[i].min(65.0);
            let y = cy + (ch as f64 * (1.0 - (val / 65.0))) as i32;
            if i == 0 {
                MoveToEx(mem_dc, x, y, null_mut());
            } else {
                LineTo(mem_dc, x, y);
            }
        }
        SelectObject(mem_dc, op);
        DeleteObject(un_pen);

        // Plot Actual Live Watts (Emerald Solid 2px)
        let live_pen = CreatePen(PS_SOLID, 2, COLOR_EMERALD);
        let op = SelectObject(mem_dc, live_pen);
        for i in 0..app.live_watts_history.len() {
            let x = cx + (i as f64 * step_x) as i32;
            let val = app.live_watts_history[i].min(65.0);
            let y = cy + (ch as f64 * (1.0 - (val / 65.0))) as i32;
            if i == 0 {
                MoveToEx(mem_dc, x, y, null_mut());
            } else {
                LineTo(mem_dc, x, y);
            }
        }
        SelectObject(mem_dc, op);
        DeleteObject(live_pen);

        // Chart Legend (14px Semi-Bold with Consistent Terminology)
        SelectObject(mem_dc, font_legend);
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_leg1 = RECT { left: 554, top: 396, right: 734, bottom: 426 };
        let leg1_str = to_wide(&format!("● Actual Draw ({:.1} W)", app.live_watts));
        DrawTextW(mem_dc, leg1_str.as_ptr(), -1, &mut rc_leg1, DT_LEFT | DT_SINGLELINE);

        SetTextColor(mem_dc, COLOR_AMBER);
        let mut rc_leg2 = RECT { left: 744, top: 396, right: 924, bottom: 426 };
        let leg2_str = to_wide(&format!("◌ Without Governor ({:.1} W)", app.unthrottled_watts));
        DrawTextW(mem_dc, leg2_str.as_ptr(), -1, &mut rc_leg2, DT_LEFT | DT_SINGLELINE);

        // =========================================================================
        // BOTTOM STATUS STRIP (Y: 468 .. 506, Height = 38 px)
        // =========================================================================
        let status_brush = CreateSolidBrush(COLOR_CARD_BG);
        let status_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_BORDER);
        SelectObject(mem_dc, status_brush);
        SelectObject(mem_dc, status_pen);
        RoundRect(mem_dc, 20, 468, 940, 506, 8, 8);
        DeleteObject(status_brush);
        DeleteObject(status_pen);

        SelectObject(mem_dc, font_status);
        SetTextColor(mem_dc, COLOR_EMERALD);
        let mut rc_foot_left = RECT { left: 36, top: 468, right: 600, bottom: 506 };
        let foot_left_str = to_wide("🛡 Autonomous Governor Active   ·   100% Baseline Guaranteed on Exit");
        DrawTextW(mem_dc, foot_left_str.as_ptr(), -1, &mut rc_foot_left, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

        SetTextColor(mem_dc, COLOR_TEXT_LABEL);
        let mut rc_foot_right = RECT { left: 600, top: 468, right: 924, bottom: 506 };
        let foot_right_str = to_wide("Sampling: 1000ms   ·   Close to Minimize to Tray");
        DrawTextW(mem_dc, foot_right_str.as_ptr(), -1, &mut rc_foot_right, DT_RIGHT | DT_VCENTER | DT_SINGLELINE);

        // =========================================================================
        // SETTINGS MODAL OVERLAY (Custom Native Input Fields: Currency & Unit Price)
        // =========================================================================
        if app.is_settings_open {
            if let Some(draft) = &app.settings_draft {
                // 1. Semi-transparent / Darkened Backdrop
                let dim_brush = CreateSolidBrush(rgb(5, 10, 18));
                let rc_dim = RECT { left: 0, top: 0, right: width, bottom: height };
                FillRect(mem_dc, &rc_dim, dim_brush);
                DeleteObject(dim_brush);

                // 2. Elevated Modal Dialog Card: X: 200..760, Y: 60..436 (Width: 560, Height: 376)
                let modal_brush = CreateSolidBrush(COLOR_CARD_BG);
                let modal_pen = CreatePen(PS_SOLID, 2, COLOR_CYAN);
                SelectObject(mem_dc, modal_brush);
                SelectObject(mem_dc, modal_pen);
                RoundRect(mem_dc, 200, 60, 760, 436, 14, 14);
                DeleteObject(modal_brush);
                DeleteObject(modal_pen);

                // Modal Title
                SelectObject(mem_dc, font_modal_title);
                SetTextColor(mem_dc, COLOR_TEXT_TITLE);
                let mut rc_m_title = RECT { left: 230, top: 74, right: 690, bottom: 98 };
                let m_title_str = to_wide("⚙ APPLICATION SETTINGS");
                DrawTextW(mem_dc, m_title_str.as_ptr(), -1, &mut rc_m_title, DT_LEFT | DT_SINGLELINE);

                // [ × ] Close Button at X: 704..744, Y: 72..104
                let close_brush = CreateSolidBrush(COLOR_TILE_BG);
                let close_pen = CreatePen(PS_SOLID, 1, COLOR_CARD_BORDER);
                SelectObject(mem_dc, close_brush);
                SelectObject(mem_dc, close_pen);
                RoundRect(mem_dc, 704, 72, 744, 104, 6, 6);
                DeleteObject(close_brush);
                DeleteObject(close_pen);

                SelectObject(mem_dc, font_card_val);
                SetTextColor(mem_dc, COLOR_TEXT_LABEL);
                let mut rc_close_btn = RECT { left: 704, top: 72, right: 744, bottom: 104 };
                let close_str = to_wide("×");
                DrawTextW(mem_dc, close_str.as_ptr(), -1, &mut rc_close_btn, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                // Subtitle Description
                SelectObject(mem_dc, font_sub);
                SetTextColor(mem_dc, COLOR_TEXT_LABEL);
                let mut rc_m_desc = RECT { left: 230, top: 100, right: 730, bottom: 120 };
                let m_desc_str = to_wide("Configure your local currency and electricity price per unit (kWh).");
                DrawTextW(mem_dc, m_desc_str.as_ptr(), -1, &mut rc_m_desc, DT_LEFT | DT_SINGLELINE);

                // Inner Divider
                let div_pen = CreatePen(PS_SOLID, 1, COLOR_DIVIDER);
                let op = SelectObject(mem_dc, div_pen);
                MoveToEx(mem_dc, 230, 128, null_mut());
                LineTo(mem_dc, 730, 128);
                SelectObject(mem_dc, op);
                DeleteObject(div_pen);

                // --- FIELD 1: CURRENCY SYMBOL / CODE ---
                SelectObject(mem_dc, font_input_lbl);
                SetTextColor(mem_dc, COLOR_TEXT_LABEL);
                let mut rc_cur_lbl = RECT { left: 230, top: 138, right: 730, bottom: 154 };
                let cur_lbl_str = to_wide("CURRENCY SYMBOL / CODE");
                DrawTextW(mem_dc, cur_lbl_str.as_ptr(), -1, &mut rc_cur_lbl, DT_LEFT | DT_SINGLELINE);

                let is_cur_active = draft.active_field == SettingsActiveField::Currency;
                let cur_pen = CreatePen(PS_SOLID, if is_cur_active { 2 } else { 1 }, if is_cur_active { COLOR_INPUT_FOCUS } else { COLOR_INPUT_BORDER });
                let cur_brush = CreateSolidBrush(COLOR_INPUT_BG);
                SelectObject(mem_dc, cur_pen);
                SelectObject(mem_dc, cur_brush);
                RoundRect(mem_dc, 230, 158, 730, 200, 6, 6);
                DeleteObject(cur_pen);
                DeleteObject(cur_brush);

                SelectObject(mem_dc, font_input_text);
                SetTextColor(mem_dc, COLOR_TEXT_BRIGHT);
                let mut rc_cur_txt = RECT { left: 244, top: 164, right: 716, bottom: 194 };
                let cur_display = if is_cur_active {
                    format!("{} |", draft.currency)
                } else if draft.currency.is_empty() {
                    "₹".to_string()
                } else {
                    draft.currency.clone()
                };
                let w_cur = to_wide(&cur_display);
                DrawTextW(mem_dc, w_cur.as_ptr(), -1, &mut rc_cur_txt, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

                SelectObject(mem_dc, font_input_hint);
                SetTextColor(mem_dc, COLOR_TEXT_MUTED);
                let mut rc_cur_hint = RECT { left: 232, top: 204, right: 730, bottom: 220 };
                let cur_hint_str = to_wide("Examples: ₹ (INR), $ (USD), € (EUR), £ (GBP), Rs, etc.");
                DrawTextW(mem_dc, cur_hint_str.as_ptr(), -1, &mut rc_cur_hint, DT_LEFT | DT_SINGLELINE);

                // --- FIELD 2: UNIT PRICE (COST PER kWh) ---
                SelectObject(mem_dc, font_input_lbl);
                SetTextColor(mem_dc, COLOR_TEXT_LABEL);
                let mut rc_price_lbl = RECT { left: 230, top: 230, right: 730, bottom: 246 };
                let price_lbl_str = to_wide("ELECTRICITY RATE (PRICE PER UNIT / kWh)");
                DrawTextW(mem_dc, price_lbl_str.as_ptr(), -1, &mut rc_price_lbl, DT_LEFT | DT_SINGLELINE);

                let is_price_active = draft.active_field == SettingsActiveField::UnitPrice;
                let price_pen = CreatePen(PS_SOLID, if is_price_active { 2 } else { 1 }, if is_price_active { COLOR_INPUT_FOCUS } else { COLOR_INPUT_BORDER });
                let price_brush = CreateSolidBrush(COLOR_INPUT_BG);
                SelectObject(mem_dc, price_pen);
                SelectObject(mem_dc, price_brush);
                RoundRect(mem_dc, 230, 250, 730, 292, 6, 6);
                DeleteObject(price_pen);
                DeleteObject(price_brush);

                SelectObject(mem_dc, font_input_text);
                SetTextColor(mem_dc, COLOR_TEXT_BRIGHT);
                let mut rc_price_txt = RECT { left: 244, top: 256, right: 716, bottom: 286 };
                let price_display = if is_price_active {
                    format!("{} |", draft.unit_price)
                } else {
                    draft.unit_price.clone()
                };
                let w_price = to_wide(&price_display);
                DrawTextW(mem_dc, w_price.as_ptr(), -1, &mut rc_price_txt, DT_LEFT | DT_VCENTER | DT_SINGLELINE);

                SelectObject(mem_dc, font_input_hint);
                SetTextColor(mem_dc, COLOR_TEXT_MUTED);
                let mut rc_price_hint = RECT { left: 232, top: 296, right: 730, bottom: 312 };
                let price_hint_str = to_wide("Enter your regional cost for 1 unit (e.g. 5.00, 7.00, 8.50)");
                DrawTextW(mem_dc, price_hint_str.as_ptr(), -1, &mut rc_price_hint, DT_LEFT | DT_SINGLELINE);

                // --- STATUS OR ERROR MESSAGE ---
                SelectObject(mem_dc, font_card_sub);
                if let Some(err) = &draft.error_msg {
                    SetTextColor(mem_dc, COLOR_RED);
                    let mut rc_msg = RECT { left: 230, top: 324, right: 730, bottom: 344 };
                    let w_msg = to_wide(&format!("⚠️ {}", err));
                    DrawTextW(mem_dc, w_msg.as_ptr(), -1, &mut rc_msg, DT_LEFT | DT_SINGLELINE);
                } else {
                    SetTextColor(mem_dc, COLOR_TEXT_MUTED);
                    let mut rc_msg = RECT { left: 230, top: 324, right: 730, bottom: 344 };
                    let w_msg = to_wide("💾 Persisted in app_settings.json  ·  Press Tab to switch, Enter to save");
                    DrawTextW(mem_dc, w_msg.as_ptr(), -1, &mut rc_msg, DT_LEFT | DT_SINGLELINE);
                }

                // --- ACTION BUTTONS ---
                // [ Cancel ] button at X: 350..470, Y: 356..398
                let cancel_b = CreateSolidBrush(COLOR_BTN_CANCEL_BG);
                let cancel_p = CreatePen(PS_SOLID, 1, COLOR_BTN_CANCEL_BORDER);
                SelectObject(mem_dc, cancel_b);
                SelectObject(mem_dc, cancel_p);
                RoundRect(mem_dc, 350, 356, 470, 398, 6, 6);
                DeleteObject(cancel_b);
                DeleteObject(cancel_p);

                SelectObject(mem_dc, font_modal_btn);
                SetTextColor(mem_dc, COLOR_TEXT_LABEL);
                let mut rc_cbtn = RECT { left: 350, top: 356, right: 470, bottom: 398 };
                let cbtn_str = to_wide("Cancel");
                DrawTextW(mem_dc, cbtn_str.as_ptr(), -1, &mut rc_cbtn, DT_CENTER | DT_VCENTER | DT_SINGLELINE);

                // [ ✓ Save Settings ] button at X: 490..730, Y: 356..398
                let save_b = CreateSolidBrush(COLOR_EMERALD_BG);
                let save_p = CreatePen(PS_SOLID, 1, COLOR_EMERALD);
                SelectObject(mem_dc, save_b);
                SelectObject(mem_dc, save_p);
                RoundRect(mem_dc, 490, 356, 730, 398, 6, 6);
                DeleteObject(save_b);
                DeleteObject(save_p);

                SelectObject(mem_dc, font_modal_btn);
                SetTextColor(mem_dc, COLOR_TEXT_TITLE);
                let mut rc_sbtn = RECT { left: 490, top: 356, right: 730, bottom: 398 };
                let sbtn_str = to_wide("✓ Save Settings");
                DrawTextW(mem_dc, sbtn_str.as_ptr(), -1, &mut rc_sbtn, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
            }
        }

        // Blit backbuffer to screen
        BitBlt(hdc, 0, 0, width, height, mem_dc, 0, 0, SRCCOPY);

        // Cleanup Fonts & GDI
        SelectObject(mem_dc, old_font);
        SelectObject(mem_dc, old_bmp);
        DeleteObject(mem_bmp);
        DeleteDC(mem_dc);

        DeleteObject(font_title);
        DeleteObject(font_sub);
        DeleteObject(font_pill);
        DeleteObject(font_sec_head);
        DeleteObject(font_metric_num);
        DeleteObject(font_metric_lbl);
        DeleteObject(font_card_val);
        DeleteObject(font_card_lbl);
        DeleteObject(font_card_sub);
        DeleteObject(font_operator);
        DeleteObject(font_active_runtime);
        DeleteObject(font_insight_head);
        DeleteObject(font_insight_body);
        DeleteObject(font_legend);
        DeleteObject(font_axis);
        DeleteObject(font_status);
        DeleteObject(font_modal_title);
        DeleteObject(font_input_text);
        DeleteObject(font_input_lbl);
        DeleteObject(font_input_hint);
        DeleteObject(font_modal_btn);
    }
}
