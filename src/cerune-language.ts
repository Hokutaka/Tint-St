import {
  HighlightStyle,
  StreamLanguage,
  syntaxHighlighting,
} from "@codemirror/language";
import { tags } from "@lezer/highlight";

const keywords =
  /^(?:import|as|pub|const|enum|match|type|fn|return|void|print|mut|if|else|while|for|in|break|continue|true|false)\b/;

const builtinTypes =
  /^(?:i8|u8|i16|u16|i32|u32|i64|u64|f32|f64|bool|string|infer)\b/;

const numberLiteral =
  /^\d+(?:\.\d+)?(?:[eE][+-]?\d+)?(?:f32|f64|i8|u8|i16|u16|i32|u32|i64|u64)?\b/;

const operatorOrPunctuation =
  /^(?:::|=>|->|==|!=|&&|\|\||<=|>=|\.\.|<<|>>|[+\-*/%=&|^~!<>:,.()\[\]{};])/;

export const ceruneLanguage = StreamLanguage.define<null>({
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

    if (stream.peek() === '"') {
      stream.next();

      let escaped = false;
      while (!stream.eol()) {
        const character = stream.next();

        if (escaped) {
          escaped = false;
          continue;
        }

        if (character === "\\") {
          escaped = true;
          continue;
        }

        if (character === '"') {
          break;
        }
      }

      return "string";
    }

    if (stream.match(builtinTypes)) {
      return "typeName";
    }

    if (stream.match(keywords)) {
      return "keyword";
    }

    if (stream.match(numberLiteral)) {
      return "number";
    }

    if (
      stream.match(
        /^[A-Za-z_][A-Za-z0-9_]*/,
      )
    ) {
      return "variableName";
    }

    if (stream.match(operatorOrPunctuation)) {
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

const ceruneHighlightStyle =
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
      tag: tags.string,
      color: "#7b8f4f",
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

export const ceruneHighlighting =
  syntaxHighlighting(
    ceruneHighlightStyle,
  );
