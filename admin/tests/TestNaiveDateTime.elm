module TestNaiveDateTime exposing (suite)

import Date
import Expect exposing (Expectation)
import Fuzz exposing (Fuzzer, intRange)
import Json.Decode as Decode
import Parser
import Test exposing (..)
import Utils.NaiveDateTime as Naive


suite : Test
suite =
    describe "DateTime"
        [ fuzz2 (Fuzz.tuple3 ( intRange 1900 2100, intRange 1 12, intRange 1 31 )) (Fuzz.tuple ( intRange 0 23, intRange 0 59 )) "date creation is consistent with other library" <|
            \( y, m, d ) ( h, min ) ->
                case Naive.build { year = y, month = m, day = d, hour = h, minute = min } of
                    Ok dateTime ->
                        let
                            referenceDate =
                                Date.fromCalendarDate y (Date.numberToMonth m) d

                            actual =
                                { year = Naive.year dateTime
                                , month = Naive.month dateTime
                                , day = Naive.day dateTime
                                , hour = Naive.hour dateTime
                                , minute = Naive.minute dateTime
                                }

                            expected =
                                { year = Date.year referenceDate
                                , month = Date.month referenceDate
                                , day = Date.day referenceDate
                                , hour = h
                                , minute = min
                                }
                        in
                        Expect.equal expected actual

                    Err _ ->
                        let
                            referenceDate =
                                Date.fromCalendarDate y (Date.numberToMonth m) d
                        in
                        Expect.notEqual (Date.day referenceDate) d
        , fuzz Naive.dateTimeFuzzer "encoding and decoding results in same DateTime" <|
            \dateTime ->
                Naive.encodeDateTime dateTime
                    |> Decode.decodeValue Naive.decodeDateTime
                    |> Expect.equal (Ok dateTime)
        ]
