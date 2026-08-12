import {
  HighlightStyle,
  StreamLanguage,
  syntaxHighlighting,
} from "@codemirror/language";
import { tags } from "@lezer/highlight";

export const primerLanguage = StreamLanguage.define<null>({
  startState() {
    return null;
  },

  token(stream) {
    if (stream.eatSpace()) {
      return null;
    }

    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }

    if (
      stream.match(
        /^(?:i64|f32|f64|infer)\b/,
      )
    ) {
      return "typeName";
    }

    if (stream.match(/^print\b/)) {
      return "keyword";
    }

    if (
      stream.match(
        /^(?:\d+\.\d+|\d+)(?:[eE][+-]?\d+)?(?:f32|f64)?\b/,
      )
    ) {
      return "number";
    }

    if (
      stream.match(
        /^[A-Za-z_][A-Za-z0-9_]*/,
      )
    ) {
      return "variableName";
    }

    if (
      stream.match(
        /^[+\-*/=:();]/,
      )
    ) {
      return "operator";
    }

    stream.next();
    return null;
  },

  languageData: {
    commentTokens: {
      line: "//",
    },
  },
});

const primerHighlightStyle =
  HighlightStyle.define([
    {
      tag: tags.keyword,
      color: "#317fa8",
      fontWeight: "600",
    },
    {
      tag: tags.typeName,
      color: "#599bc0",
      fontWeight: "600",
    },
    {
      tag: tags.number,
      color: "#8b72b1",
    },
    {
      tag: tags.comment,
      color: "#9badb8",
      fontStyle: "italic",
    },
    {
      tag: tags.variableName,
      color: "#263746",
    },
    {
      tag: tags.operator,
      color: "#6d8492",
    },
  ]);

export const primerHighlighting =
  syntaxHighlighting(
    primerHighlightStyle,
  );
  