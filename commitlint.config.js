module.exports = {
  extends: ["@commitlint/config-conventional"],
  rules: {
    "scope-enum": [
      2,
      "always",
      [
        "core",
        "c",
        "core",
        "cpp",
        "csharp",
        "deno",
        "go",
        "java",
        "node",
        "php",
        "python",
        "ruby",
        "rust",
      ],
    ],
  },
};
