module Utils.Validate exposing (Validator, accept, errors, from, ifEmpty, ifNotInt, map2, validate)


type Validator raw a
    = SimpleValidator (raw -> Result a)
    | CombinedValidator ( Validator raw a, List (Validator raw a) ) (( a, List a ) -> a)


type alias Result a =
    Result.Result (List String) a


validate : Validator raw a -> raw -> Result a
validate validator input =
    case validator of
        SimpleValidator doValidation ->
            doValidation input

        CombinedValidator ( first, validators ) merge ->
            List.map (\val -> validate val input) validators
                |> List.foldl
                    (\next accu ->
                        case next of
                            Ok v ->
                                Result.map (Tuple.mapSecond (List.append [ v ])) accu

                            Err errs ->
                                case accu of
                                    Ok _ ->
                                        Err errs

                                    Err existing ->
                                        Err (existing ++ errs)
                    )
                    (validate first input |> Result.map (\val -> ( val, [] )))
                |> Result.map merge


map2 : (a -> b -> c) -> Result a -> Result b -> Result c
map2 mapping first second =
    case ( first, second ) of
        ( Ok v1, Ok v2 ) ->
            Ok (mapping v1 v2)

        ( Err error, Ok _ ) ->
            Err error

        ( Ok _, Err error ) ->
            Err error

        ( Err e1, Err e2 ) ->
            Err (e1 ++ e2)


errors : Validator raw a -> raw -> List String
errors validator input =
    case validate validator input of
        Ok _ ->
            []

        Err errs ->
            errs


from : (raw -> Result a) -> Validator raw a
from validator =
    SimpleValidator validator


accept : Validator raw raw
accept =
    from Ok


ifEmpty : String -> Validator String String
ifEmpty error =
    SimpleValidator
        (\raw ->
            if String.length raw > 0 then
                Ok raw

            else
                Err [ error ]
        )


ifNotInt : String -> Validator String Int
ifNotInt error =
    SimpleValidator
        (\raw ->
            case String.toInt raw of
                Just n ->
                    Ok n

                Nothing ->
                    Err [ error ]
        )
