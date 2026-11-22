//! 검사 유형: 관계형(합성 공개 API의 저장 상자와 그림 사이 일관성); 절대 페이지 좌표·golden 핀 없음.
//! 입력: 빈 문서 "가에" 사이에 HYhwpEQ 수식을 새로 넣거나(insert), 짧은 수식의 스크립트를 고친다(edit).
//! 기대 관계: 수식 뒤 글자 "에"는 수식이 그린 마지막 글리프의 시작점보다 오른쪽에서 시작한다.
//!
//! 새로 만든 수식의 저장 폭은 종전에 일반 layout(`EqLayout::new`)으로 재고, 그림은 HYhwpEQ
//! layout(`EqLayout::for_equation`)으로 그렸다. `f(x)=(x+2)(2x^2 -x-2)` 은 저장 11274 대
//! 그림 12100 HWPUNIT 이라, 줄 흐름이 저장 폭만큼만 전진해 뒤 글자가 수식 끝 `2)` 를 덮었다.
//! 한/글 2020 은 같은 파일에서 수식을 그린 폭만큼 띄운다(합성 입력의 한컴 Print raw 는 없다).

use rhwp::document_core::DocumentCore;

const SCRIPTS: [&str; 3] = [
    "f(x)=(x+2)(2x^2 -x-2)",
    "y=- {1} over {5} x^2 +3",
    "log _a ab",
];

fn attr_f64(tag: &str, name: &str) -> Option<f64> {
    let start = tag.find(&format!(" {name}=\""))? + name.len() + 3;
    let rest = &tag[start..];
    rest[..rest.find('"')?].parse().ok()
}

/// (수식 그룹 원점 x, 그 안 마지막 글리프의 시작 x(절대), 뒤 글자 "에" 의 x)
fn equation_and_next_glyph(svg: &str) -> (f64, f64, f64) {
    let eq_at = svg.find("font-family=\"HyhwpEQ\"").expect("수식 글리프");
    let group_at = svg[..eq_at]
        .rfind("<g transform=\"translate(")
        .expect("수식 그룹");
    let group_rest = &svg[group_at + "<g transform=\"translate(".len()..];
    let origin_x: f64 = group_rest[..group_rest.find(',').unwrap()].parse().unwrap();
    let group_end = group_at + svg[group_at..].find("</g>").expect("수식 그룹 끝");
    let last_glyph_x = svg[group_at..group_end]
        .split("matrix(1 0 -0 1 ")
        .skip(1)
        .filter_map(|tail| tail.split(' ').next()?.parse::<f64>().ok())
        .fold(f64::MIN, f64::max);
    let next_tag_end = svg[group_end..].find(">에</text>").expect("뒤 글자") + group_end;
    let next_tag = &svg[svg[..next_tag_end].rfind("<text").unwrap()..next_tag_end];
    let next_x = attr_f64(next_tag, "x").expect("뒤 글자 x");
    (origin_x, origin_x + last_glyph_x, next_x)
}

fn assert_next_glyph_clears_equation(core: &DocumentCore, script: &str) {
    let svg = core.render_page_svg_native(0).unwrap();
    let (origin, last_glyph, next) = equation_and_next_glyph(&svg);
    assert!(
        next > last_glyph,
        "{script}: 뒤 글자({next:.2})가 수식 마지막 글리프 시작({last_glyph:.2}) 앞에 놓였다 — 수식 원점 {origin:.2}"
    );
}

fn blank_with_text() -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    core.insert_text_native(0, 0, 0, "가에").unwrap();
    core
}

#[test]
fn inserted_equation_reserves_its_painted_width() {
    for script in SCRIPTS {
        let mut core = blank_with_text();
        core.insert_equation_native(0, 0, 1, script, 1000, 0)
            .unwrap();
        assert_next_glyph_clears_equation(&core, script);
    }
}

#[test]
fn edited_equation_script_reserves_its_painted_width() {
    for script in SCRIPTS {
        let mut core = blank_with_text();
        let inserted: serde_json::Value =
            serde_json::from_str(&core.insert_equation_native(0, 0, 1, "x", 1000, 0).unwrap())
                .unwrap();
        let control = inserted["controlIdx"].as_u64().expect("수식 컨트롤") as usize;
        core.set_equation_properties_native(
            0,
            0,
            control,
            None,
            None,
            &serde_json::json!({ "script": script }).to_string(),
        )
        .unwrap();
        assert_next_glyph_clears_equation(&core, script);
    }
}
