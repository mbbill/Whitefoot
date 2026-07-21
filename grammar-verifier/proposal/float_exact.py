"""Exact rational IEEE-754 RNE and Whitefoot canonical-decimal search.

The implementation uses integers and ``fractions.Fraction`` only.  It never
parses, formats, or rounds through a host floating-point value.
"""

from __future__ import annotations

from dataclasses import dataclass
from fractions import Fraction
import re


@dataclass(frozen=True)
class BinaryFormat:
    name: str
    exponent_bits: int
    fraction_bits: int
    bias: int

    @property
    def sign_mask(self) -> int:
        return 1 << (self.exponent_bits + self.fraction_bits)

    @property
    def exponent_mask(self) -> int:
        return (1 << self.exponent_bits) - 1

    @property
    def fraction_mask(self) -> int:
        return (1 << self.fraction_bits) - 1

    @property
    def maximum_finite_magnitude(self) -> int:
        return ((self.exponent_mask - 1) << self.fraction_bits) | self.fraction_mask

    @property
    def infinity_magnitude(self) -> int:
        return self.exponent_mask << self.fraction_bits

    @property
    def byte_width(self) -> int:
        return (1 + self.exponent_bits + self.fraction_bits) // 8


F32 = BinaryFormat("f32", 8, 23, 127)
F64 = BinaryFormat("f64", 11, 52, 1023)
FORMATS = {item.name: item for item in (F32, F64)}


@dataclass(frozen=True)
class RoundingInterval:
    lower: Fraction
    lower_inclusive: bool
    upper: Fraction
    upper_inclusive: bool


@dataclass(frozen=True)
class DecimalValue:
    format: BinaryFormat
    negative: bool
    magnitude: Fraction


_FLOAT_LITERAL = re.compile(
    r"(-?)(0|[1-9][0-9]*)\.([0-9]+)(?:e(-?)(0|[1-9][0-9]*))?_(f32|f64)\Z"
)


def _power_of_two(exponent: int) -> Fraction:
    if exponent >= 0:
        return Fraction(1 << exponent, 1)
    return Fraction(1, 1 << -exponent)


def _power_of_ten(exponent: int) -> Fraction:
    if exponent >= 0:
        return Fraction(10**exponent, 1)
    return Fraction(1, 10**-exponent)


def finite_magnitude_value(binary_format: BinaryFormat, magnitude_bits: int) -> Fraction:
    """Decode one finite signless bit pattern to an exact nonnegative rational."""

    if not 0 <= magnitude_bits <= binary_format.maximum_finite_magnitude:
        raise ValueError("bit pattern is not finite")
    exponent_field = magnitude_bits >> binary_format.fraction_bits
    fraction = magnitude_bits & binary_format.fraction_mask
    if exponent_field == 0:
        significand = fraction
        exponent = 1 - binary_format.bias - binary_format.fraction_bits
    else:
        significand = (1 << binary_format.fraction_bits) | fraction
        exponent = exponent_field - binary_format.bias - binary_format.fraction_bits
    return significand * _power_of_two(exponent)


def finite_value(binary_format: BinaryFormat, bits: int) -> Fraction:
    """Decode one finite signed pattern; negative zero becomes rational zero."""

    negative = bool(bits & binary_format.sign_mask)
    magnitude = finite_magnitude_value(binary_format, bits & ~binary_format.sign_mask)
    return -magnitude if negative and magnitude else magnitude


def _significand_is_even(binary_format: BinaryFormat, magnitude_bits: int) -> bool:
    exponent_field = magnitude_bits >> binary_format.fraction_bits
    fraction = magnitude_bits & binary_format.fraction_mask
    significand = fraction if exponent_field == 0 else (1 << binary_format.fraction_bits) | fraction
    return significand % 2 == 0


def _hypothetical_after_maximum(binary_format: BinaryFormat) -> Fraction:
    maximum_unbiased_exponent = binary_format.exponent_mask - 1 - binary_format.bias
    return _power_of_two(maximum_unbiased_exponent + 1)


def rounding_interval(binary_format: BinaryFormat, bits: int) -> RoundingInterval:
    """Return the exact nonnegative RNE interval for one finite signed pattern."""

    magnitude_bits = bits & ~binary_format.sign_mask
    if magnitude_bits > binary_format.maximum_finite_magnitude:
        raise ValueError("interval requires a finite bit pattern")
    value = finite_magnitude_value(binary_format, magnitude_bits)
    even = _significand_is_even(binary_format, magnitude_bits)
    if magnitude_bits == 0:
        previous = Fraction(0)
    else:
        previous = finite_magnitude_value(binary_format, magnitude_bits - 1)
    if magnitude_bits == binary_format.maximum_finite_magnitude:
        following = _hypothetical_after_maximum(binary_format)
    else:
        following = finite_magnitude_value(binary_format, magnitude_bits + 1)
    return RoundingInterval(
        Fraction(0) if magnitude_bits == 0 else (previous + value) / 2,
        True if magnitude_bits == 0 else even,
        (value + following) / 2,
        even,
    )


def round_rational(
    binary_format: BinaryFormat,
    magnitude: Fraction,
    *,
    negative: bool = False,
) -> int:
    """Round an exact nonnegative rational under IEEE RNE to bits."""

    if magnitude < 0:
        raise ValueError("magnitude must be nonnegative")
    sign = binary_format.sign_mask if negative else 0
    if magnitude == 0:
        return sign
    low = 0
    high = binary_format.maximum_finite_magnitude
    while low < high:
        middle = (low + high + 1) // 2
        if finite_magnitude_value(binary_format, middle) <= magnitude:
            low = middle
        else:
            high = middle - 1
    lower_bits = low
    lower_value = finite_magnitude_value(binary_format, lower_bits)
    if lower_bits == binary_format.maximum_finite_magnitude:
        upper_bits = binary_format.infinity_magnitude
        upper_value = _hypothetical_after_maximum(binary_format)
    else:
        upper_bits = lower_bits + 1
        upper_value = finite_magnitude_value(binary_format, upper_bits)
    lower_distance = magnitude - lower_value
    upper_distance = upper_value - magnitude
    if lower_distance < upper_distance:
        selected = lower_bits
    elif upper_distance < lower_distance:
        selected = upper_bits
    else:
        lower_even = _significand_is_even(binary_format, lower_bits)
        upper_even = (
            True
            if upper_bits == binary_format.infinity_magnitude
            else _significand_is_even(binary_format, upper_bits)
        )
        if lower_even == upper_even:
            raise AssertionError("adjacent RNE candidates do not have opposite parity")
        selected = lower_bits if lower_even else upper_bits
    return sign | selected


def parse_decimal_literal(source: str) -> DecimalValue:
    """Parse the exact successor grammar without a host decimal or float parser."""

    match = _FLOAT_LITERAL.fullmatch(source)
    if match is None:
        raise ValueError("float literal does not match the successor grammar")
    sign, integer, fraction, exponent_sign, exponent_digits, type_name = match.groups()
    negative = sign == "-"
    exponent = 0 if exponent_digits is None else int(exponent_digits)
    if exponent_sign == "-":
        exponent = -exponent
    digits = integer + fraction
    if all(digit == "0" for digit in digits):
        magnitude = Fraction(0)
    else:
        coefficient = int(digits)
        magnitude = coefficient * _power_of_ten(exponent - len(fraction))
    return DecimalValue(FORMATS[type_name], negative, magnitude)


def rounded_literal_bits(source: str) -> int:
    parsed = parse_decimal_literal(source)
    return round_rational(parsed.format, parsed.magnitude, negative=parsed.negative)


def _least_integer(bound: Fraction, inclusive: bool) -> int:
    quotient, remainder = divmod(bound.numerator, bound.denominator)
    if remainder:
        return quotient + 1
    return quotient if inclusive else quotient + 1


def _greatest_integer(bound: Fraction, inclusive: bool) -> int:
    quotient, remainder = divmod(bound.numerator, bound.denominator)
    if remainder:
        return quotient
    return quotient if inclusive else quotient - 1


def _floor_log10(value: Fraction) -> int:
    if value <= 0:
        raise ValueError("decimal magnitude requires a positive rational")
    estimate = len(str(value.numerator)) - len(str(value.denominator))
    while value < _power_of_ten(estimate):
        estimate -= 1
    while value >= _power_of_ten(estimate + 1):
        estimate += 1
    return estimate


def _candidate_for_structure(
    interval: RoundingInterval,
    *,
    negative: bool,
    integer_digits: int,
    fraction_digits: int,
    exponent: int,
    exponent_present: bool,
    negative_exponent_spelling: bool,
) -> str | None:
    digit_count = integer_digits + fraction_digits
    scale = _power_of_ten(exponent - fraction_digits)
    lower = interval.lower / scale
    upper = interval.upper / scale
    minimum = _least_integer(lower, interval.lower_inclusive)
    maximum = _greatest_integer(upper, interval.upper_inclusive)
    minimum_digits = 0 if integer_digits == 1 else 10 ** (digit_count - 1)
    maximum_digits = 10**digit_count - 1
    coefficient = max(minimum, minimum_digits)
    if coefficient > min(maximum, maximum_digits):
        return None
    digits = str(coefficient).zfill(digit_count)
    prefix = ("-" if negative else "") + digits[:integer_digits] + "." + digits[integer_digits:]
    if exponent_present:
        exponent_magnitude = abs(exponent)
        prefix += "e" + ("-" if negative_exponent_spelling else "") + str(exponent_magnitude)
    return prefix


def canonical_spelling(
    binary_format: BinaryFormat,
    bits: int,
    *,
    maximum_prefix_bytes: int = 64,
) -> str:
    """Select the exact minimum-byte, ASCII-lexicographic finite spelling."""

    magnitude_bits = bits & ~binary_format.sign_mask
    if magnitude_bits > binary_format.maximum_finite_magnitude:
        raise ValueError("canonical spelling requires a finite pattern")
    negative = bool(bits & binary_format.sign_mask)
    if magnitude_bits == 0:
        # These are already the grammar's absolute minimum prefix lengths (three
        # bytes, or four with the value sign). Every same-length nonzero fixed
        # decimal has magnitude at least 0.1, while adding an exponent is longer.
        result = ("-" if negative else "") + "0.0_" + binary_format.name
        if rounded_literal_bits(result) != bits:
            raise AssertionError("signed zero spelling does not round to its target")
        return result
    interval = rounding_interval(binary_format, bits)
    decimal_low = _floor_log10(interval.lower)
    decimal_high = _floor_log10(interval.upper)
    sign_bytes = int(negative)
    for prefix_bytes in range(sign_bytes + 3, maximum_prefix_bytes + 1):
        candidates: list[str] = []
        for integer_digits in range(1, prefix_bytes + 1):
            fraction_digits = prefix_bytes - sign_bytes - integer_digits - 1
            if fraction_digits >= 1:
                candidate = _candidate_for_structure(
                    interval,
                    negative=negative,
                    integer_digits=integer_digits,
                    fraction_digits=fraction_digits,
                    exponent=0,
                    exponent_present=False,
                    negative_exponent_spelling=False,
                )
                if candidate is not None:
                    candidates.append(candidate)
            for negative_exponent_spelling in (False, True):
                exponent_sign_bytes = int(negative_exponent_spelling)
                for exponent_digits in range(1, prefix_bytes + 1):
                    fraction_digits = (
                        prefix_bytes
                        - sign_bytes
                        - integer_digits
                        - 1
                        - 1
                        - exponent_sign_bytes
                        - exponent_digits
                    )
                    if fraction_digits < 1:
                        continue
                    coefficient_digits = integer_digits + fraction_digits
                    exponent_minimum = decimal_low - (coefficient_digits - 1) + fraction_digits
                    exponent_maximum = decimal_high + fraction_digits
                    for exponent in range(exponent_minimum, exponent_maximum + 1):
                        if negative_exponent_spelling:
                            if exponent > 0:
                                continue
                            magnitude = -exponent
                        else:
                            if exponent < 0:
                                continue
                            magnitude = exponent
                        if len(str(magnitude)) != exponent_digits:
                            continue
                        candidate = _candidate_for_structure(
                            interval,
                            negative=negative,
                            integer_digits=integer_digits,
                            fraction_digits=fraction_digits,
                            exponent=exponent,
                            exponent_present=True,
                            negative_exponent_spelling=negative_exponent_spelling,
                        )
                        if candidate is not None:
                            candidates.append(candidate)
        if candidates:
            selected = min(candidates)
            result = selected + "_" + binary_format.name
            if len(selected.encode("ascii")) != prefix_bytes:
                raise AssertionError("canonical search length drifted")
            if rounded_literal_bits(result) != bits:
                raise AssertionError("canonical spelling does not round to its target")
            return result
    raise ValueError("no canonical spelling found within the declared evidence bound")


def bits_hex(binary_format: BinaryFormat, bits: int) -> str:
    return bits.to_bytes(binary_format.byte_width, "big").hex()
