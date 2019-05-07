module Pages.Utils exposing
    ( Enabledness(..)
    , In
    , Input
    , breadcrumbs
    , buildInput
    , button
    , buttonWithOptions
    , dateTimeValidator
    , extract
    , fields
    , inputDateTime
    , inputString
    , labeled
    , updateInput
    , validate
    , viewDateTimeInput
    , viewInputNumber
    , viewInputText
    , viewSelection
    , viewTextArea
    )

import Css exposing (center, column, em, flexStart, none, row, zero)
import Css.Global as Css
import Html.Styled as Html exposing (Html, a, div, input, label, li, nav, ol, text, textarea)
import Html.Styled.Attributes exposing (css, disabled, href, type_, value)
import Html.Styled.Events exposing (onClick, onInput)
import Parser
import Routes exposing (Route)
import Utils.NaiveDateTime as Naive
import Utils.TimeFormat as TimeFormat
import Utils.Validate as Validate exposing (Validator)



-- Navigation


breadcrumbs : List Route -> Route -> Html msg
breadcrumbs routes current =
    let
        entriesHtml =
            List.map
                (\route ->
                    a [ href (Routes.toRelativeUrl <| route) ] [ text <| Routes.routeName route ]
                )
                routes
                ++ [ text <| Routes.routeName current ]

        breadcrumbStyle =
            Css.batch
                [ Css.listStyleType none
                , Css.padding zero
                , Css.displayFlex
                , Css.flexDirection row
                , Css.children
                    [ Css.typeSelector "li"
                        [ Css.adjacentSiblings
                            [ Css.typeSelector "li"
                                [ Css.marginLeft (em 0.5)
                                , Css.before
                                    [ Css.property "content" "\">\""
                                    , Css.marginRight (em 0.5)
                                    ]
                                ]
                            ]
                        ]
                    ]
                ]
    in
    nav []
        [ ol [ css [ breadcrumbStyle ] ]
            (List.map
                (\entryHtml ->
                    li [] [ entryHtml ]
                )
                entriesHtml
            )
        ]



-- Forms


type Input raw a
    = Input raw (Validator raw a)


buildInput : raw -> Validator raw a -> Input raw a
buildInput =
    Input


updateInput : (raw -> raw) -> Input raw a -> Input raw a
updateInput mapping (Input raw validator) =
    Input (mapping raw) validator


validate : Input raw a -> Result (List String) a
validate (Input raw validator) =
    Validate.validate validator raw


extract : Input raw a -> Maybe a
extract input =
    validate input |> Result.toMaybe


type alias In a =
    Input String a


inputString : String -> In String
inputString value =
    buildInput value (Validate.ifEmpty "Darf nicht leer sein.")


inputDateTime : Naive.DateTime -> Input { date : String, time : String } Naive.DateTime
inputDateTime dateTime =
    let
        value =
            { date = Naive.encodeDateAsString dateTime, time = Naive.encodeTimeAsString dateTime }
    in
    buildInput value dateTimeValidator


dateTimeValidator : Validator { date : String, time : String } Naive.DateTime
dateTimeValidator =
    Validate.from
        (\{ date, time } ->
            let
                dateResult =
                    Parser.run Naive.dateParser date |> Result.mapError (\err -> [ "Das Datum ist ungültig." ])

                timeResult =
                    Parser.run Naive.timeParser time |> Result.mapError (\err -> [ "Die Uhrzeit ist ungültig." ])
            in
            Validate.map2
                Naive.with
                dateResult
                timeResult
        )


viewInputText : String -> In a -> (String -> msg) -> Html msg
viewInputText lbl (Input val validator) inputMsg =
    labeled lbl
        ([ input [ type_ "text", value val, onInput inputMsg ] [] ]
            ++ viewErrors (Input val validator)
        )


viewInputNumber : String -> In a -> (String -> msg) -> Html msg
viewInputNumber lbl (Input val validator) inputMsg =
    labeled lbl
        ([ input [ type_ "number", value val, onInput inputMsg ] [] ]
            ++ viewErrors (Input val validator)
        )


viewTextArea : String -> In a -> (String -> msg) -> Html msg
viewTextArea lbl (Input val validator) inputMsg =
    labeled lbl
        ([ textarea [ value val, onInput inputMsg ] []
         ]
            ++ viewErrors (Input val validator)
        )


viewSelection : String -> In a -> List { name : String, value : String } -> (String -> msg) -> Html msg
viewSelection lbl (Input val validator) options inputMsg =
    let
        optionsHtml =
            List.map
                (\option ->
                    let
                        selected =
                            Html.Styled.Attributes.selected (option.value == val)
                    in
                    Html.option [ value option.value, selected ] [ text option.name ]
                )
                options

        optionsWithEmpty =
            if List.any (\option -> val == option.value) options then
                optionsHtml

            else
                [ Html.option [] [ text "Bitte wählen..." ] ] ++ optionsHtml
    in
    labeled lbl
        [ Html.select [ value val, onInput inputMsg ] optionsWithEmpty
        ]


viewDateTimeInput :
    String
    -> Input { date : String, time : String } Naive.DateTime
    -> { dateChanged : String -> msg, timeChanged : String -> msg }
    -> Html msg
viewDateTimeInput lbl (Input { date, time } validator) toMsgs =
    labeled lbl
        ([ input [ type_ "date", value date, onInput toMsgs.dateChanged ] []
         , input [ type_ "time", value time, onInput toMsgs.timeChanged ] []
         ]
            ++ viewErrors (Input { date = date, time = time } validator)
        )


viewErrors : Input raw a -> List (Html msg)
viewErrors (Input val validator) =
    let
        errors =
            Validate.errors validator val
    in
    case errors of
        [] ->
            []

        _ ->
            [ ol [] (List.map (\error -> li [] [ text error ]) errors) ]


labeled : String -> List (Html msg) -> Html msg
labeled lbl content =
    label [ css [ labelStyle ] ] (text lbl :: content)


labelStyle : Css.Style
labelStyle =
    Css.batch
        [ Css.displayFlex
        , Css.flexDirection column
        , Css.alignItems flexStart
        ]


fields : List (Html msg) -> Html msg
fields content =
    div [ css [ Css.children [ Css.everything [ inputSpacingStyle ] ] ] ]
        content


inputSpacingStyle : Css.Style
inputSpacingStyle =
    Css.batch
        [ Css.marginTop (em 1)
        , Css.marginBottom (em 1)
        ]


button =
    buttonWithOptions { enabledness = Enabled }


type Enabledness
    = Enabled
    | Disabled


buttonWithOptions : { enabledness : Enabledness } -> String -> msg -> Html msg
buttonWithOptions options lbl msg =
    let
        isDisabled =
            case options.enabledness of
                Enabled ->
                    False

                Disabled ->
                    True
    in
    Html.button [ onClick msg, disabled isDisabled ] [ text lbl ]
