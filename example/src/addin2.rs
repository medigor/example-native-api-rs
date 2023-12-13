use addin1c::{name, MethodInfo, Methods, ParamValue, PropInfo, SimpleAddin, Variant};

pub struct Addin2 {
    prop1: i32,
}

impl Addin2 {
    pub fn new() -> Addin2 {
        Addin2 { prop1: 0 }
    }

    fn method1(&mut self, param: &mut Variant, ret_value: &mut Variant) -> bool {
        let ParamValue::I32(value) = param.get() else {
            return false;
        };
        self.prop1 = value;
        ret_value.set_i32(value * 2);
        true
    }

    fn method2(
        &mut self,
        param1: &mut Variant,
        param2: &mut Variant,
        ret_value: &mut Variant,
    ) -> bool {
        let ParamValue::I32(value1) = param1.get() else {
            return false;
        };
        let ParamValue::I32(value2) = param2.get() else {
            return false;
        };
        self.prop1 = value1 + value2;
        ret_value.set_i32(self.prop1);
        true
    }

    fn set_prop1(&mut self, value: &ParamValue) -> bool {
        let ParamValue::I32(value) = value else {
            return false;
        };
        self.prop1 = *value;
        true
    }

    fn get_prop1(&mut self, value: &mut Variant) -> bool {
        value.set_i32(self.prop1);
        true
    }
}

impl SimpleAddin for Addin2 {
    fn name() -> &'static [u16] {
        name!("Class2")
    }

    fn methods() -> &'static [MethodInfo<Self>] {
        &[
            MethodInfo {
                name: name!("Method1"),
                method: Methods::Method1(Self::method1),
            },
            MethodInfo {
                name: name!("Method2"),
                method: Methods::Method2(Self::method2),
            },
        ]
    }

    fn properties() -> &'static [PropInfo<Self>] {
        &[PropInfo {
            name: name!("Prop1"),
            getter: Some(Self::get_prop1),
            setter: Some(Self::set_prop1),
        }]
    }
}
