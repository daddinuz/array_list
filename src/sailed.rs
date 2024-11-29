use std::mem::MaybeUninit;

pub trait Array<T, const N: usize> {
    fn uninit_array() -> [MaybeUninit<T>; N] {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

impl<T> Array<T, 1> for [T; 1] {}
impl<T> Array<T, 2> for [T; 2] {}
impl<T> Array<T, 3> for [T; 3] {}
impl<T> Array<T, 4> for [T; 4] {}
impl<T> Array<T, 5> for [T; 5] {}
impl<T> Array<T, 6> for [T; 6] {}
impl<T> Array<T, 7> for [T; 7] {}
impl<T> Array<T, 8> for [T; 8] {}
impl<T> Array<T, 9> for [T; 9] {}
impl<T> Array<T, 10> for [T; 10] {}
impl<T> Array<T, 11> for [T; 11] {}
impl<T> Array<T, 12> for [T; 12] {}
impl<T> Array<T, 13> for [T; 13] {}
impl<T> Array<T, 14> for [T; 14] {}
impl<T> Array<T, 15> for [T; 15] {}
impl<T> Array<T, 16> for [T; 16] {}
impl<T> Array<T, 17> for [T; 17] {}
impl<T> Array<T, 18> for [T; 18] {}
impl<T> Array<T, 19> for [T; 19] {}
impl<T> Array<T, 20> for [T; 20] {}
impl<T> Array<T, 21> for [T; 21] {}
impl<T> Array<T, 22> for [T; 22] {}
impl<T> Array<T, 23> for [T; 23] {}
impl<T> Array<T, 24> for [T; 24] {}
impl<T> Array<T, 25> for [T; 25] {}
impl<T> Array<T, 26> for [T; 26] {}
impl<T> Array<T, 27> for [T; 27] {}
impl<T> Array<T, 28> for [T; 28] {}
impl<T> Array<T, 29> for [T; 29] {}
impl<T> Array<T, 30> for [T; 30] {}
impl<T> Array<T, 31> for [T; 31] {}
impl<T> Array<T, 32> for [T; 32] {}
impl<T> Array<T, 33> for [T; 33] {}
impl<T> Array<T, 34> for [T; 34] {}
impl<T> Array<T, 35> for [T; 35] {}
impl<T> Array<T, 36> for [T; 36] {}
impl<T> Array<T, 37> for [T; 37] {}
impl<T> Array<T, 38> for [T; 38] {}
impl<T> Array<T, 39> for [T; 39] {}
impl<T> Array<T, 40> for [T; 40] {}
impl<T> Array<T, 41> for [T; 41] {}
impl<T> Array<T, 42> for [T; 42] {}
impl<T> Array<T, 43> for [T; 43] {}
impl<T> Array<T, 44> for [T; 44] {}
impl<T> Array<T, 45> for [T; 45] {}
impl<T> Array<T, 46> for [T; 46] {}
impl<T> Array<T, 47> for [T; 47] {}
impl<T> Array<T, 48> for [T; 48] {}
impl<T> Array<T, 49> for [T; 49] {}
impl<T> Array<T, 50> for [T; 50] {}
impl<T> Array<T, 51> for [T; 51] {}
impl<T> Array<T, 52> for [T; 52] {}
impl<T> Array<T, 53> for [T; 53] {}
impl<T> Array<T, 54> for [T; 54] {}
impl<T> Array<T, 55> for [T; 55] {}
impl<T> Array<T, 56> for [T; 56] {}
impl<T> Array<T, 57> for [T; 57] {}
impl<T> Array<T, 58> for [T; 58] {}
impl<T> Array<T, 59> for [T; 59] {}
impl<T> Array<T, 60> for [T; 60] {}
impl<T> Array<T, 61> for [T; 61] {}
impl<T> Array<T, 62> for [T; 62] {}
impl<T> Array<T, 63> for [T; 63] {}
impl<T> Array<T, 64> for [T; 64] {}
