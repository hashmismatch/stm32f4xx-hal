pub mod rate {
    mod units {
        use core::cmp;
        pub trait MyTryFrom<T>: Sized {
            type Error;
            fn try_from(value: T) -> Result<Self, Self::Error>;
        }
        pub trait MyTryInto<T>: Sized {
            type Error;
            fn try_into(self) -> Result<T, Self::Error>;
        }
        impl<T, U> MyTryInto<U> for T
        where
            U: MyTryFrom<T>,
        {
            type Error = U::Error;

            fn try_into(self) -> Result<U, U::Error> {
                U::try_from(self)
            }
        }
        /// Conversion errors
        #[non_exhaustive]
        #[derive(Debug, Eq, PartialEq, Hash)]
        pub enum ConversionError {
            /// Exact cause of failure is unknown
            Unspecified,
            /// Attempted type conversion failed
            ConversionFailure,
            /// Result is outside of those valid for this type
            Overflow,
            /// Attempted to divide by zero
            DivByZero,
            /// Resulting [`Duration`](duration/trait.Duration.html) is negative (not allowed)
            NegDuration,
        }

        pub const fn gcd_u64(mut m: u64, mut n: u64) -> u64 {
            // Use Stein's algorithm
            if m == 0 || n == 0 {
                return m | n;
            }

            // find common factors of 2
            let shift = (m | n).trailing_zeros();

            // divide n and m by 2 until odd
            m >>= m.trailing_zeros();
            n >>= n.trailing_zeros();

            while m != n {
                if m > n {
                    m -= n;
                    m >>= m.trailing_zeros();
                } else {
                    n -= m;
                    n >>= n.trailing_zeros();
                }
            }
            m << shift
        }

        pub trait TimeInt: From<u32> + PartialEq + Copy {}

        impl TimeInt for u32 {}
        impl TimeInt for u64 {}

        #[derive(Copy, Clone, Debug, Default)]
        pub struct Unit<U, const N: u32, const D: u32>(U);

        impl<U, const N: u32, const D: u32> Unit<U, N, D> {
            pub const fn new(val: U) -> Self {
                Self(val)
            }
        }
        impl<U, const N: u32, const D: u32> Unit<U, N, D> {
            pub fn integer(self) -> U {
                self.0
            }
        }

        impl<U, const N: u32, const D: u32> Unit<U, N, D> {
            pub const fn scale_factor<const N2: u32, const D2: u32>() -> Option<(u32, u32)> {
                let (num, denum) = Self::scale_factor_u64::<N2, D2>();
                if num > u32::MAX as u64 || denum > u32::MAX as u64 {
                    None
                } else {
                    Some((num as u32, denum as u32))
                }
            }
            pub const fn scale_factor_u64<const N2: u32, const D2: u32>() -> (u64, u64) {
                let num = (N2 as u64) * (D as u64);
                let denum = (D2 as u64) * (N as u64);
                let gcd = gcd_u64(num, denum);
                let num = num / gcd;
                let denum = denum / gcd;
                (num, denum)
            }
        }
        impl<const N: u32, const D: u32, const N2: u32, const D2: u32> MyTryFrom<Unit<u32, N2, D2>>
            for Unit<u64, N, D>
        {
            type Error = ConversionError;
            fn try_from(f: Unit<u32, N2, D2>) -> Result<Self, Self::Error> {
                let val = f.0 as u64;
                let (num, denum) = Self::scale_factor_u64::<N2, D2>();
                if num == 1 && denum == 1 {
                    Ok(Self(val))
                } else {
                    if let Some(up) = val.checked_mul(num) {
                        Ok(Self(up / denum))
                    //} else if let Some(down) = val.checked_div(denum) {
                    //    Self(down * num)
                    } else {
                        Err(ConversionError::Overflow)
                    }
                }
            }
        }
        impl<const N: u32, const D: u32, const N2: u32, const D2: u32> MyTryFrom<Unit<u64, N2, D2>>
            for Unit<u64, N, D>
        {
            type Error = ConversionError;
            fn try_from(f: Unit<u64, N2, D2>) -> Result<Self, Self::Error> {
                let (num, denum) = Self::scale_factor_u64::<N2, D2>();
                if num == 1 && denum == 1 {
                    Ok(Self(f.0))
                } else {
                    if let Some(up) = f.0.checked_mul(num) {
                        Ok(Self(up / denum))
                    } else {
                        Err(ConversionError::Overflow)
                    }
                }
            }
        }
        impl<const N: u32, const D: u32, const N2: u32, const D2: u32> MyTryFrom<Unit<u64, N2, D2>>
            for Unit<u32, N, D>
        {
            type Error = ConversionError;
            fn try_from(f: Unit<u64, N2, D2>) -> Result<Self, Self::Error> {
                let (num, denum) = Self::scale_factor_u64::<N2, D2>();
                if num == 1 && denum == 1 {
                    let res_u64 = f.0;
                    if res_u64 > u32::MAX as u64 {
                        Err(ConversionError::Overflow)
                    } else {
                        Ok(Self(res_u64 as u32))
                    }
                } else {
                    if let Some(up) = f.0.checked_mul(num) {
                        let res_u64 = up / denum;
                        if res_u64 > u32::MAX as u64 {
                            Err(ConversionError::Overflow)
                        } else {
                            Ok(Self(res_u64 as u32))
                        }
                    } else {
                        Err(ConversionError::Overflow)
                    }
                }
            }
        }
        impl<const N: u32, const D: u32, const N2: u32, const D2: u32> MyTryFrom<Unit<u32, N2, D2>>
            for Unit<u32, N, D>
        {
            type Error = ConversionError;
            fn try_from(f: Unit<u32, N2, D2>) -> Result<Self, Self::Error> {
                if let Some((num, denum)) = Self::scale_factor::<N2, D2>() {
                    if num == 1 && denum == 1 {
                        Ok(Self(f.0))
                    } else {
                        if let Some(up) = f.0.checked_mul(num) {
                            Ok(Self(up / denum))
                        } else {
                            Err(ConversionError::Overflow)
                        }
                    }
                } else {
                    Err(ConversionError::ConversionFailure)
                }
            }
        }

        impl<
                T: TimeInt,
                RhsInt: TimeInt,
                const N: u32,
                const D: u32,
                const N2: u32,
                const D2: u32,
            > cmp::PartialEq<Unit<RhsInt, N2, D2>> for Unit<T, N, D>
        where
            Self: MyTryFrom<Unit<RhsInt, N2, D2>>,
        {
            fn eq(&self, rhs: &Unit<RhsInt, N2, D2>) -> bool {
                match Self::try_from(*rhs) {
                    Ok(rhs) => self.integer() == rhs.integer(),
                    Err(_) => false,
                }
            }
        }
        /*
        impl<T: TimeInt, RhsInt: TimeInt> PartialOrd<$name<RhsInt>> for $name<T>
        where
            T: TryFrom<RhsInt>,
        {
            /// See [Comparisons](trait.Rate.html#comparisons)
            fn partial_cmp(&self, rhs: &$name<RhsInt>) -> Option<core::cmp::Ordering> {
                match T::try_from(rhs.integer()) {
                    Ok(rhs_integer) => Some(self.integer().cmp(&rhs_integer)),
                    Err(_) => Some(core::cmp::Ordering::Less),
                }
            }
        }*/
    }

    use core::convert::TryFrom;
    use units::*;

    #[derive(Copy, Clone, Debug, Default)]
    pub struct Hertz<T = u32>(Unit<T, 1, 1>);
    #[derive(Copy, Clone, Debug, Default)]
    pub struct Kilohertz<T = u32>(Unit<T, 1_000, 1>);
    #[derive(Copy, Clone, Debug, Default)]
    pub struct Megahertz<T = u32>(Unit<T, 1_000_000, 1>);

    impl<T> Hertz<T> {
        pub const fn new(val: T) -> Self {
            Self(Unit::new(val))
        }
        pub fn integer(self) -> T {
            self.0.integer()
        }
    }

    impl<T: TimeInt> PartialEq for Hertz<T>
    where
        Unit<T, 1, 1>: PartialEq<Unit<T, 1, 1>>,
    {
        fn eq(&self, rhs: &Self) -> bool {
            self.0.eq(&rhs.0)
        }
    }

    impl<T> Kilohertz<T> {
        pub const fn new(val: T) -> Self {
            Self(Unit::new(val))
        }
        pub fn integer(self) -> T {
            self.0.integer()
        }
    }

    impl<T> Megahertz<T> {
        pub const fn new(val: T) -> Self {
            Self(Unit::new(val))
        }
        pub fn integer(self) -> T {
            self.0.integer()
        }
    }

    // impl From<$small<u32>> for $big<u64>
    impl<T, T2> From<Hertz<T2>> for Kilohertz<T>
    where
        Unit<T, 1_000, 1>: MyTryFrom<Unit<T2, 1, 1>>,
    {
        fn from(small: Hertz<T2>) -> Self {
            Self(small.0.try_into().unwrap_or_else(|_| panic!("From failed")))
        }
    }

    // impl From<$small<u32>> for $big<u64>
    impl<T, T2> From<Hertz<T2>> for Megahertz<T>
    where
        Unit<T, 1_000_000, 1>: MyTryFrom<Unit<T2, 1, 1>>,
    {
        fn from(small: Hertz<T2>) -> Self {
            Self(small.0.try_into().unwrap_or_else(|_| panic!("From failed")))
        }
    }

    // impl From<$small<u32>> for $big<u64>
    impl<T, T2> From<Kilohertz<T2>> for Megahertz<T>
    where
        Unit<T, 1_000_000, 1>: MyTryFrom<Unit<T2, 1_000, 1>>,
    {
        fn from(small: Kilohertz<T2>) -> Self {
            Self(small.0.try_into().unwrap_or_else(|_| panic!("From failed")))
        }
    }

    //impl TryFrom<$big<u64>> for $small<u32>
    impl<T, T2> TryFrom<Kilohertz<T2>> for Hertz<T>
    where
        Unit<T, 1, 1>: MyTryFrom<Unit<T2, 1_000, 1>, Error = ConversionError>,
    {
        type Error = ConversionError;
        fn try_from(big: Kilohertz<T2>) -> Result<Self, Self::Error> {
            big.0.try_into().map(Self)
        }
    }

    //impl TryFrom<$big<u64>> for $small<u32>
    impl<T, T2> TryFrom<Megahertz<T2>> for Hertz<T>
    where
        Unit<T, 1, 1>: MyTryFrom<Unit<T2, 1_000_000, 1>, Error = ConversionError>,
    {
        type Error = ConversionError;
        fn try_from(big: Megahertz<T2>) -> Result<Self, Self::Error> {
            big.0.try_into().map(Self)
        }
    }

    //impl TryFrom<$big<u64>> for $small<u32>
    impl<T, T2> TryFrom<Megahertz<T2>> for Kilohertz<T>
    where
        Unit<T, 1_000, 1>: MyTryFrom<Unit<T2, 1_000_000, 1>, Error = ConversionError>,
    {
        type Error = ConversionError;
        fn try_from(big: Megahertz<T2>) -> Result<Self, Self::Error> {
            big.0.try_into().map(Self)
        }
    }

    pub trait Extensions: TimeInt {
        /*        /// mebihertz
                fn MiHz(self) -> Mebihertz<Self> {
                    Mebihertz::new(self)
                }
        */
        /// megahertz
        fn MHz(self) -> Megahertz<Self> {
            Megahertz::new(self)
        }
        /*
                /// kibihertz
                fn KiHz(self) -> Kibihertz<Self> {
                    Kibihertz::new(self)
                }
        */
        /// kilohertz
        fn kHz(self) -> Kilohertz<Self> {
            Kilohertz::new(self)
        }

        /// hertz
        fn Hz(self) -> Hertz<Self> {
            Hertz::new(self)
        }
        /*
        /// mebibytes per second
        fn MiBps(self) -> MebibytesPerSecond<Self> {
            MebibytesPerSecond::new(self)
        }

        /// megabytes per second
        fn MBps(self) -> MegabytesPerSecond<Self> {
            MegabytesPerSecond::new(self)
        }

        /// kibibytes per second
        fn KiBps(self) -> KibibytesPerSecond<Self> {
            KibibytesPerSecond::new(self)
        }

        /// kiloBytes per second
        fn kBps(self) -> KilobytesPerSecond<Self> {
            KilobytesPerSecond::new(self)
        }

        /// bytes per second
        fn Bps(self) -> BytesPerSecond<Self> {
            BytesPerSecond::new(self)
        }

        /// mebibits per second
        fn Mibps(self) -> MebibitsPerSecond<Self> {
            MebibitsPerSecond::new(self)
        }

        /// megabits per second
        fn Mbps(self) -> MegabitsPerSecond<Self> {
            MegabitsPerSecond::new(self)
        }

        /// kibibits per second
        fn Kibps(self) -> KibibitsPerSecond<Self> {
            KibibitsPerSecond::new(self)
        }

        /// kilobits per second
        fn kbps(self) -> KilobitsPerSecond<Self> {
            KilobitsPerSecond::new(self)
        }

        /// bits per second
        fn bps(self) -> BitsPerSecond<Self> {
            BitsPerSecond::new(self)
        }

        /// mebibaud
        fn MiBd(self) -> Mebibaud<Self> {
            Mebibaud::new(self)
        }

        /// megabaud
        fn MBd(self) -> Megabaud<Self> {
            Megabaud::new(self)
        }

        /// kibibaud
        fn KiBd(self) -> Kibibaud<Self> {
            Kibibaud::new(self)
        }

        /// kilobaud
        fn kBd(self) -> Kilobaud<Self> {
            Kilobaud::new(self)
        }

        /// baud
        fn Bd(self) -> Baud<Self> {
            Baud::new(self)
        }*/
    }

    impl Extensions for u32 {}
}
