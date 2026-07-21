use iced::advanced::widget::operation::Outcome;
use iced::advanced::widget::{Id, Operation};
use iced::Rectangle;

pub struct MeasurePopupContent {
    target_id: Id,
    result: Option<f32>,
}

impl MeasurePopupContent {
    pub fn new(target_id: Id) -> Self {
        Self {
            target_id,
            result: None,
        }
    }
}

impl Operation<f32> for MeasurePopupContent {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<f32>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        let matched = id == Some(&self.target_id);
        if matched && self.result.is_none() {
            self.result = Some(bounds.height);
        }
    }

    fn finish(&self) -> Outcome<f32> {
        match self.result {
            Some(height) => Outcome::Some(height),
            None => Outcome::None,
        }
    }
}
