use super::defaults::get_init_border;
use super::manager::Manager;
use crate::payloads::sheet_process::style::BorderPayloadType;
use xlrs_workbook::simple_types::StBorderStyle;
use xlrs_workbook::styles::*;

pub type BorderId = u32;
pub type BorderManager = Manager<Border, BorderId>;

impl Default for BorderManager {
    fn default() -> Self {
        let border = get_init_border();
        let mut manager = BorderManager::new(0);
        manager.get_id(border);
        manager
    }
}

impl BorderManager {
    pub fn execute(self, payload: &BorderPayload) -> (Self, BorderId) {
        let mut res = self.clone();
        let base = payload.id;
        if let Some(border) = res.get_data(base) {
            let mut new_border = border.clone();
            handle(&mut new_border, payload.change.clone());
            let new_id = res.get_id(new_border);
            (res, new_id)
        } else {
            (res, 0)
        }
    }
}

pub struct BorderPayload {
    pub id: BorderId,
    pub change: BorderPayloadType,
}

fn handle(border: &mut Border, ty: BorderPayloadType) {
    use xlrs_workbook::complex_types::Color;
    match ty {
        BorderPayloadType::LeftBorderColor(s) => match &mut border.left {
            Some(pr) => match &mut pr.color {
                Some(c) => c.rgb = Some(s),
                None => {
                    let mut color = Color::new_with_rgb(s);
                    pr.color = Some(color);
                }
            },
            None => {
                let mut color = Color::new_with_rgb(s);
                border.left = Some(BorderPr {
                    color: Some(color),
                    style: StBorderStyle::Type::None,
                });
            }
        },
        BorderPayloadType::RightBorderColor(s) => match &mut border.right {
            Some(pr) => match &mut pr.color {
                Some(c) => c.rgb = Some(s),
                None => {
                    let mut color = Color::new_with_rgb(s);
                    pr.color = Some(color);
                }
            },
            None => {
                let mut color = Color::new_with_rgb(s);
                border.right = Some(BorderPr {
                    color: Some(color),
                    style: StBorderStyle::Type::None,
                });
            }
        },
        BorderPayloadType::TopBorderColor(s) => match &mut border.top {
            Some(pr) => match &mut pr.color {
                Some(c) => c.rgb = Some(s),
                None => {
                    let mut color = Color::new_with_rgb(s);
                    pr.color = Some(color);
                }
            },
            None => {
                let mut color = Color::new_with_rgb(s);
                border.top = Some(BorderPr {
                    color: Some(color),
                    style: StBorderStyle::Type::None,
                });
            }
        },
        BorderPayloadType::BottomBorderColor(s) => match &mut border.bottom {
            Some(pr) => match &mut pr.color {
                Some(c) => c.rgb = Some(s),
                None => {
                    let mut color = Color::new_with_rgb(s);
                    pr.color = Some(color);
                }
            },
            None => {
                let mut color = Color::new_with_rgb(s);
                border.bottom = Some(BorderPr {
                    color: Some(color),
                    style: StBorderStyle::Type::None,
                });
            }
        },
        BorderPayloadType::LeftBorderStyle(s) => match &mut border.left {
            Some(pr) => pr.style = s,
            None => {
                border.left = Some(BorderPr {
                    color: None,
                    style: s,
                })
            }
        },
        BorderPayloadType::RightBorderStyle(s) => match &mut border.right {
            Some(pr) => pr.style = s,
            None => {
                border.right = Some(BorderPr {
                    color: None,
                    style: s,
                })
            }
        },
        BorderPayloadType::TopBorderStyle(s) => match &mut border.top {
            Some(pr) => pr.style = s,
            None => {
                border.top = Some(BorderPr {
                    color: None,
                    style: s,
                })
            }
        },
        BorderPayloadType::BottomBorderStyle(s) => match &mut border.bottom {
            Some(pr) => pr.style = s,
            None => {
                border.bottom = Some(BorderPr {
                    color: None,
                    style: s,
                })
            }
        },
        BorderPayloadType::BorderDiagonalUp(b) => {
            border.diagonal_up = Some(b);
        }
        BorderPayloadType::BorderDiagonalDown(b) => {
            border.diagonal_down = Some(b);
        }
        BorderPayloadType::Outline(b) => {
            border.outline = b;
        }
    };
}
