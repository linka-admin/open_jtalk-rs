use super::*;
use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
    os::raw::{c_char, c_void},
};

extern "C" {
    // `NJD_clear`/`NJD_refresh`が`free`で解放するため、ノードは`calloc`で確保する必要がある
    fn calloc(count: usize, size: usize) -> *mut c_void;
}

#[derive(Default)]
pub struct Njd(Option<open_jtalk_sys::NJD>);

/// NJDノード1つ分の特徴量。pyopenjtalkの`NJDFeature`に相当する。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NjdFeature {
    pub string: String,
    pub pos: String,
    pub pos_group1: String,
    pub pos_group2: String,
    pub pos_group3: String,
    pub ctype: String,
    pub cform: String,
    pub orig: String,
    pub read: String,
    pub pron: String,
    pub acc: i32,
    pub mora_size: i32,
    pub chain_rule: String,
    pub chain_flag: i32,
}

unsafe impl resources::Resource for Njd {
    unsafe fn initialize(&mut self) -> bool {
        if self.0.is_some() {
            panic!("njd already initialized");
        }
        let njd = {
            let mut njd = MaybeUninit::<open_jtalk_sys::NJD>::uninit();
            open_jtalk_sys::NJD_initialize(njd.as_mut_ptr());
            njd.assume_init()
        };
        self.0 = Some(njd);
        true
    }
    unsafe fn clear(&mut self) -> bool {
        open_jtalk_sys::NJD_clear(self.as_raw_ptr());
        self.0 = None;
        true
    }
}

// SAFETY: `Send`と対立する性質はないはず。
unsafe impl Send for Njd {}

impl Njd {
    pub(crate) unsafe fn as_raw_ptr(&self) -> *mut open_jtalk_sys::NJD {
        if self.0.is_none() {
            panic!("uninitialized njd");
        }
        self.0.as_ref().unwrap() as *const open_jtalk_sys::NJD as *mut open_jtalk_sys::NJD
    }

    pub fn set_pronunciation(&mut self) {
        unsafe { open_jtalk_sys::njd_set_pronunciation(self.as_raw_ptr()) }
    }

    pub fn set_digit(&mut self) {
        unsafe { open_jtalk_sys::njd_set_digit(self.as_raw_ptr()) }
    }

    pub fn set_accent_type(&mut self) {
        unsafe { open_jtalk_sys::njd_set_accent_type(self.as_raw_ptr()) }
    }

    pub fn set_accent_phrase(&mut self) {
        unsafe { open_jtalk_sys::njd_set_accent_phrase(self.as_raw_ptr()) }
    }

    pub fn set_unvoiced_vowel(&mut self) {
        unsafe { open_jtalk_sys::njd_set_unvoiced_vowel(self.as_raw_ptr()) }
    }

    pub fn set_long_vowel(&mut self) {
        unsafe { open_jtalk_sys::njd_set_long_vowel(self.as_raw_ptr()) }
    }

    /// 全ノードの特徴量を取り出す。
    pub fn features(&self) -> Vec<NjdFeature> {
        let mut features = Vec::new();
        unsafe {
            let mut node = (*self.as_raw_ptr()).head;
            while !node.is_null() {
                features.push(NjdFeature {
                    string: to_string(open_jtalk_sys::NJDNode_get_string(node)),
                    pos: to_string(open_jtalk_sys::NJDNode_get_pos(node)),
                    pos_group1: to_string(open_jtalk_sys::NJDNode_get_pos_group1(node)),
                    pos_group2: to_string(open_jtalk_sys::NJDNode_get_pos_group2(node)),
                    pos_group3: to_string(open_jtalk_sys::NJDNode_get_pos_group3(node)),
                    ctype: to_string(open_jtalk_sys::NJDNode_get_ctype(node)),
                    cform: to_string(open_jtalk_sys::NJDNode_get_cform(node)),
                    orig: to_string(open_jtalk_sys::NJDNode_get_orig(node)),
                    read: to_string(open_jtalk_sys::NJDNode_get_read(node)),
                    pron: to_string(open_jtalk_sys::NJDNode_get_pron(node)),
                    acc: open_jtalk_sys::NJDNode_get_acc(node),
                    mora_size: open_jtalk_sys::NJDNode_get_mora_size(node),
                    chain_rule: to_string(open_jtalk_sys::NJDNode_get_chain_rule(node)),
                    chain_flag: open_jtalk_sys::NJDNode_get_chain_flag(node),
                });
                node = (*node).next;
            }
        }
        return features;

        unsafe fn to_string(s: *const c_char) -> String {
            if s.is_null() {
                String::new()
            } else {
                CStr::from_ptr(s).to_string_lossy().into_owned()
            }
        }
    }

    /// 全ノードを`features`で置き換える。
    ///
    /// # Panics
    ///
    /// 文字列にNUL文字が含まれているとパニックする。
    pub fn set_features(&mut self, features: &[NjdFeature]) {
        unsafe {
            let njd = self.as_raw_ptr();
            open_jtalk_sys::NJD_refresh(njd);
            for feature in features {
                let node = calloc(1, std::mem::size_of::<open_jtalk_sys::NJDNode>())
                    as *mut open_jtalk_sys::NJDNode;
                assert!(!node.is_null(), "failed to allocate NJDNode");
                open_jtalk_sys::NJDNode_initialize(node);
                let c = |s: &str| CString::new(s).expect("should not contain NUL");
                open_jtalk_sys::NJDNode_set_string(node, c(&feature.string).as_ptr());
                open_jtalk_sys::NJDNode_set_pos(node, c(&feature.pos).as_ptr());
                open_jtalk_sys::NJDNode_set_pos_group1(node, c(&feature.pos_group1).as_ptr());
                open_jtalk_sys::NJDNode_set_pos_group2(node, c(&feature.pos_group2).as_ptr());
                open_jtalk_sys::NJDNode_set_pos_group3(node, c(&feature.pos_group3).as_ptr());
                open_jtalk_sys::NJDNode_set_ctype(node, c(&feature.ctype).as_ptr());
                open_jtalk_sys::NJDNode_set_cform(node, c(&feature.cform).as_ptr());
                open_jtalk_sys::NJDNode_set_orig(node, c(&feature.orig).as_ptr());
                open_jtalk_sys::NJDNode_set_read(node, c(&feature.read).as_ptr());
                open_jtalk_sys::NJDNode_set_pron(node, c(&feature.pron).as_ptr());
                open_jtalk_sys::NJDNode_set_acc(node, feature.acc);
                open_jtalk_sys::NJDNode_set_mora_size(node, feature.mora_size);
                open_jtalk_sys::NJDNode_set_chain_rule(node, c(&feature.chain_rule).as_ptr());
                open_jtalk_sys::NJDNode_set_chain_flag(node, feature.chain_flag);
                open_jtalk_sys::NJD_push_node(njd, node);
            }
        }
    }

    pub fn refresh(&mut self) {
        unsafe { open_jtalk_sys::NJD_refresh(self.as_raw_ptr()) }
    }

    pub fn mecab2njd(&mut self, mecab_feature: &MecabFeature, mecab_feature_size: i32) {
        unsafe {
            open_jtalk_sys::mecab2njd(
                self.as_raw_ptr(),
                mecab_feature as *const MecabFeature as *mut *mut c_char,
                mecab_feature_size,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8Path;
    use resources::Resource as _;
    #[rstest]
    fn njd_initialize_and_clear_works() {
        let mut njd = Njd::default();
        unsafe {
            assert!(njd.initialize());
            assert!(njd.clear());
        }
    }

    #[rstest]
    fn njd_set_pronunciation_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_pronunciation();
    }

    #[rstest]
    fn njd_set_digit_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_digit();
    }

    #[rstest]
    fn njd_set_accent_type_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_accent_type();
    }

    #[rstest]
    fn njd_set_accent_phrase_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_accent_phrase();
    }

    #[rstest]
    fn njd_set_unvoiced_vowel_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_unvoiced_vowel();
    }
    #[rstest]
    fn njd_set_long_vowel_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.set_long_vowel();
    }
    #[rstest]
    fn njd_refresh_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        njd.refresh();
    }

    #[rstest]
    fn njd_mecab2njd_works() {
        let mut njd = ManagedResource::<Njd>::initialize();
        let mut mecab = ManagedResource::<Mecab>::initialize();

        mecab
            .load(
                Utf8Path::new(std::env!("CARGO_MANIFEST_DIR"))
                    .join("src/mecab/testdata/mecab_load"),
            )
            .unwrap();
        let s = text2mecab("h^o-d+e=s/A:2+3+2/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4@2+1&2-1|6+4/J:xx_xx/K:2+2-9").unwrap();
        assert!(mecab.analysis(s));
        njd.mecab2njd(mecab.get_feature().unwrap(), mecab.get_size());
    }

    fn feature(string: &str, pron: &str, acc: i32, chain_flag: i32) -> NjdFeature {
        NjdFeature {
            string: string.to_owned(),
            pos: "名詞".to_owned(),
            pos_group1: "一般".to_owned(),
            pos_group2: "*".to_owned(),
            pos_group3: "*".to_owned(),
            ctype: "*".to_owned(),
            cform: "*".to_owned(),
            orig: string.to_owned(),
            read: pron.to_owned(),
            pron: pron.to_owned(),
            acc,
            mora_size: 2,
            chain_rule: "C1".to_owned(),
            chain_flag,
        }
    }

    #[rstest]
    fn njd_features_is_empty_after_initialize() {
        let njd = ManagedResource::<Njd>::initialize();
        assert_eq!(Vec::<NjdFeature>::new(), njd.features());
    }

    #[rstest]
    fn njd_set_features_round_trips() {
        let mut njd = ManagedResource::<Njd>::initialize();
        let features = vec![feature("春", "ハル", 1, -1), feature("風", "カゼ", 0, 1)];
        njd.set_features(&features);
        assert_eq!(features, njd.features());

        // 置き換えになる
        let features = vec![feature("空", "ソラ", 1, -1)];
        njd.set_features(&features);
        assert_eq!(features, njd.features());
    }

    #[rstest]
    fn njd_features_reflect_mecab2njd() {
        let mut njd = ManagedResource::<Njd>::initialize();
        let mut mecab = ManagedResource::<Mecab>::initialize();
        mecab
            .load(
                Utf8Path::new(std::env!("CARGO_MANIFEST_DIR"))
                    .join("src/mecab/testdata/mecab_load"),
            )
            .unwrap();
        let s = text2mecab("h^o-d+e=s/A:2+3+2/B:22-xx_xx/C:10_7+2/D:xx+xx_xx/E:5_5!0_xx-0/F:4_1#0_xx@1_1|1_4/G:xx_xx%xx_xx_xx/H:1_5/I:1-4@2+1&2-1|6+4/J:xx_xx/K:2+2-9").unwrap();
        assert!(mecab.analysis(s));
        njd.mecab2njd(mecab.get_feature().unwrap(), mecab.get_size());

        let features = njd.features();
        assert!(!features.is_empty());
        assert!(features.iter().all(|f| !f.string.is_empty()));

        // 書き戻しても変わらない
        njd.set_features(&features);
        assert_eq!(features, njd.features());
    }
}
