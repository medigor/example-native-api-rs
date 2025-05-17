use addin1c::{name, AddinResult, CStr1C, MethodInfo, Methods, PropInfo, SimpleAddin, Variant};

pub struct Addin2 {
    prop1: i32,
}

impl Addin2 {
    pub fn new() -> Addin2 {
        Addin2 { prop1: 0 }
    }

    fn method1(&mut self, param: &mut Variant, ret_value: &mut Variant) -> AddinResult {
        let value = param.get_i32()?;
        self.prop1 = value;
        ret_value.set_i32(value * 2);
        Ok(())
    }

    fn method2(
        &mut self,
        param1: &mut Variant,
        param2: &mut Variant,
        ret_value: &mut Variant,
    ) -> AddinResult {
        let value1 = param1.get_i32()?;
        let value2 = param2.get_i32()?;
        self.prop1 = value1 + value2;
        ret_value.set_i32(self.prop1);
        Ok(())
    }

    fn set_prop1(&mut self, value: &Variant) -> AddinResult {
        self.prop1 = value.get_i32()?;
        Ok(())
    }

    fn get_prop1(&mut self, value: &mut Variant) -> AddinResult {
        value.set_i32(self.prop1);
        Ok(())
    }
}

impl SimpleAddin for Addin2 {
    fn name() -> &'static CStr1C {
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
