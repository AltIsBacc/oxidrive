use crate::{engine::{buffer::AudioBuffer, streams::ResolvedStreamConfig}, traits::normalized::Normalized};

pub mod chain;
pub mod commands;
pub mod graph;

macro_rules! generate_node_controls {
    ($(
        $field:ident : $t:ty = $default:expr
        $( => $normalize:expr )?
    );* $(;)?) => {
        pub struct NodeControlsBase {
            $( $field: $t, )*
        }

        impl Default for NodeControlsBase {
            fn default() -> Self {
                Self {
                    $( $field: $default, )*
                }
            }
        }

        impl NodeControlsBase {
            pastey::paste! {
                $(
                    #[inline]
                    pub fn $field(&self) -> $t {
                        self.$field
                    }

                    #[inline]
                    pub fn [<set_ $field>](&mut self, val: $t) {
                        #[allow(clippy::redundant_closure_call)]
                        let val = generate_node_controls!(@apply val $(, $normalize)?);
                        self.$field = val;
                    }
                )*
            }
        }

        pub trait NodeControls {
            fn controls(&self) -> &NodeControlsBase;
            fn controls_mut(&mut self) -> &mut NodeControlsBase;

            pastey::paste! {
                $(
                    #[inline]
                    fn $field(&self) -> $t {
                        self.controls().$field()
                    }

                    #[inline]
                    fn [<set_ $field>](&mut self, val: $t) {
                        self.controls_mut().[<set_ $field>](val)
                    }
                )*
            }

            #[inline]
            fn should_process(&self) -> bool {
                !self.bypass() && self.output_gain().to_bool() && self.mix() != 0.0
            }
        }
    };

    (@apply $val:ident) => { $val };
    (@apply $val:ident, $normalize:expr) => { ($normalize)($val) };
}

generate_node_controls! {
    bypass:      bool = 1.0.to_bool();
    input_gain:  f32  = 1.0 => |v: f32| v.max(0.0);
    mix:         f32  = 1.0 => |v: f32| v.clamp(0.0, 1.0);
    output_gain: f32  = 1.0 => |v: f32| v.clamp(0.0, 5.0);
}

pub trait PedalNode: Send + NodeControls {
    fn prepare(&mut self, config: &ResolvedStreamConfig);
    fn process(&mut self, data: &mut AudioBuffer<'_, f32>);
    fn name(&self) -> &str;
    fn set_param_raw(&mut self, param: u32, value: f32);
}

pub trait PedalNodeExt: PedalNode {
    fn set_param<P: Into<u32>>(&mut self, param: P, value: f32) {
        self.set_param_raw(param.into(), value);
    }
}

impl<T: PedalNode + ?Sized> PedalNodeExt for T {}

type BoxedPedal = Box<dyn PedalNode>;

