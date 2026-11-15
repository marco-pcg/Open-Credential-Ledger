# Paper boilerplate

Portuguese LaTeX academic draft with cover, title page, approval/cataloguing placeholders, resumo/abstract, contents, research sections, references and appendix. Author, advisor, co-advisor, degree, course, campus and submission year are unconfirmed placeholders. No invented affiliation approval, results or signatures.

Compile with a full TeX Live installation:

```sh
cd paper
pdflatex -interaction=nonstopmode -halt-on-error main.tex
pdflatex -interaction=nonstopmode -halt-on-error main.tex
```

Requires babel Portuguese, newtx and geometry. The A4 margins (top/left 3 cm, bottom/right 2 cm), 12 pt text and 1.5 spacing use the [CEFET/RJ Petrópolis 2022 manual](https://www.cefet-rj.br/attachments/article/9308/Manual%20TCC%203%20ed%20revisado%20e%20atualizado%202022.pdf) as a starting point. Course/campus rules, current ABNT requirements, citations, numbering, approval sheet and library catalogue must be confirmed before submission. This is editable source, not a certified institutional template.

Research context: [W3C VC Data Model 2.0](https://www.w3.org/TR/vc-data-model-2.0/). This prototype does not implement that standard. Framework implementation references: [Axum routing](https://docs.rs/axum/latest/axum/struct.Router.html), [Alloy contract calls](https://alloy.rs/contract-interactions/using-sol%21/), [SQLx migrations](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html).
