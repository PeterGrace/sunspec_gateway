#[allow(clippy::use_self)]
impl ::core::str::FromStr for
PWRCellUnitClass
{
    type Err = ::strum::ParseError;
    fn from_str(s: &str) -> ::core::result::Result<PWRCellUnitClass, <Self as ::core::str::FromStr>::Err>
    {
        ::core::result::Result::
        Ok(match s
        {
            s if s.eq_ignore_ascii_case("rebus beacon") => PWRCellUnitClass::
            Beacon,
            s if s.eq_ignore_ascii_case("ICM") => PWRCellUnitClass::
            ICM,
            s if s.eq_ignore_ascii_case("PWRcell XVT076A03 Inverter") =>
                PWRCellUnitClass::Inverter,
            s if
            s.eq_ignore_ascii_case("PWRcell Battery") => PWRCellUnitClass::
            Battery,
            s if s.eq_ignore_ascii_case("PV Link") =>
                PWRCellUnitClass::PVLink,
            _ => return ::core::result::
            Result::Err(::strum::ParseError::VariantNotFound),
        })
    }
}

#[allow(clippy::use_self)]
impl ::core::convert::TryFrom<&str>
for PWRCellUnitClass
{
    type Error = ::strum::ParseError;
    fn try_from(s: &str) -> ::core::
    result::Result<PWRCellUnitClass, <Self as ::core::convert::
    TryFrom<&str>>::Error> { ::core::str::FromStr::from_str(s) }
}
warning: unused import: `ModelData`
