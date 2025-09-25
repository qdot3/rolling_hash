//! Rolling Hashの試験的実装
//!
//! ## 基本機能
//!
//! - 高速に剰余計算ができる特別な素数を法に使用（~2^60）
//! - 実行時にランダムな基数を生成
//!
//! - `Vec`のように末尾で値の追加・削除ができる。
//! - 元の数列を持っておく必要はない。
//! 連続部分列のハッシュ値を計算するには、ハッシュ値の列があれば十分。
//! 元の数列の型がバラバラでも問題ない？
//!
//! ## メモ
//!
//! - `u64`以外の値を受け取れるようにする
//!
//! - 回文などでは同じ法・同じ基数の組・同じ値の列について、逆向きにハッシュを計算したい。
//! あとから計算するためには元の数列を保持しておく必要がある。
//! このとき、`Vec`のように振舞うことはできず、むしろ`[_]`に近い。
//! なので、`OneWay`と`TowWay`を用意し、内部的には後者は前者を２つ持てばよい。
//!
//! - `std`に倣って抽象化する必要があるか？
//!   - 向きがあるので、一方通行と双方向のものだけあればよい。
//!   - １つの法といくつかの基数についてRolling Hashを計算できれば、それを組み合わせればよい。
//!   - `Hash`は値の拡張性のために必要。
//! 自動的に`Hasher`も必要
//!
//! - 基数を複数もつときは`[Vec<_>; B]`よりも`Vec<[_; B]>`の方が早期リターンしやすくて良い。
//! キャッシュ効率もよさそう。
//! - 複数の基数に対するハッシュ値をまとめることもできるが不要。
//! 基数の数に比例してき計算時間が増大するので、少数に制限するのが良く、arrayで十分。
use std::ops::Deref;

mod prime;
pub use prime::{PRIMES, Prime, SupportedPrime};

mod oneway;
pub use oneway::OneWay;

mod convert;
pub use convert::Reduce;

pub(crate) mod mock;
pub(crate) use mock::cold_path;

pub(crate) mod windows;
pub(crate) use windows::Windows;

/// Specifies the number of bases in [`RollingHasher`].
///
/// This sill be small.
pub struct BaseCount<const B: usize>;

/// A marker trait for supported number of bases.
pub trait SupportedBaseCount {}

macro_rules! supported_base_count_impl {
    ($( $b:literal ),+) => {$(
        impl SupportedBaseCount for BaseCount<{ $b }> {}
    )+};
}
supported_base_count_impl! { 2, 3, 4, 5, 6, 7, 8, 9, 10 }

/// A value that may be incorrect due to hash collisions.
pub struct Maybe<T>(T);

impl<T> Deref for Maybe<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
