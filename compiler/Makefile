PY=python3 -B

check:
	$(PY) test_lexer.py
	$(PY) test_parser.py
	$(PY) test_parser_items.py
	$(PY) test_parser_expressions.py
	$(PY) test_parser_control.py
	$(PY) test_ast_validate.py
	$(PY) test_self_parse.py
	$(PY) test_output.py
	$(PY) test_source_names.py
	$(PY) test_symbols.py
	$(PY) test_semantic_prelude.py
	$(PY) test_semantic_types.py
	$(PY) test_semantic_globals.py
	$(PY) test_semantic_type_resolve.py
	$(PY) test_semantic_all_types.py
	$(PY) test_semantic_function_symbols.py
	$(PY) test_semantic_type_members.py
	$(PY) test_frontend.py

.PHONY: check
