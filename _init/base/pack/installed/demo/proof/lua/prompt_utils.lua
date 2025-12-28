local PROMPT_TEMPLATE = [[
```toml
#!meta
# model name. e.g., gpt-5.2, gemini-3-flash-preview,
# or aliases (from ~/.aipack-base/config-default.toml)
model = "gpt" # alias to gpt-5.2
```

Do not add em dash (—), use a comma. Do not use dashes as sentence separators, never.

Do not change the line layout, and keep single empty lines.

Make it sound American.

====
> Placeholder: Replace this line and below with your content to be proofread.
]]

-- Returns FileRecord
function prep_prompt_file(input, options)
	options                   = options or {}
	local default_prompt_path = options.default_prompt_path
	local placeholder_suffix  = options.placeholder_suffix
	local initial_content     = options.initial_content
	local add_separator       = options.add_separator ~= nil and options.add_separator or false

	-- Enter default file_stem
	local prompt_path         = nil
	if input == nil then
		prompt_path = default_prompt_path
	elseif type(input) == "string" then
		-- remove the trailing /
		prompt_path = input:gsub("/+$", "")
		prompt_path = aip.text.ensure(input, { prefix = "./", suffix = "/prompt.md" })
	else
		prompt_path = input.path
	end

	-- Get flag
	local first_time = aip.path.exists(prompt_path) ~= true

	-- Create placeholder initial content
	-- (otherwise, the initial content will be)
	if placeholder_suffix ~= nil then
		initial_content = PROMPT_TEMPLATE
	else
		if initial_content == nil then
			initial_content = ""
		end
	end

	aip.file.ensure_exists(prompt_path, initial_content, { content_when_empty = true })

	-- DISABLE for now (Should support use future `aip.editor.open_file(...)`)
	-- if first_time then
	-- 	ok, err = pcall(aip.cmd.exec, "code", { prompt_path })
	-- end

	return aip.file.load(prompt_path)
end

-- Will return a aipack skip if this task should be skipped
--   - If both inst and content are empty
--   - Or if inst (or content if inst is empty) starts with 'placeholder'
function should_skip(inst, content)
	inst = inst and aip.text.trim(inst) or ""
	content = content and aip.text.trim(content) or ""

	if inst == "" and content == "" then
		return aip.skip("Empty content and instructions - Start writing, and do a redo.")
	end

	if content == "" then
		return aip.flow.skip("Content is empty, start writting content")
	end

	-- if starts with placeholder
	if content and content:sub(1, 13):lower() == "> placeholder" then
		return aip.flow.skip("Content is a placeholder, so skipping for now")
	end

	return nil
end

-- returns `inst, content` and each can be nil
-- options {content_is_default = bool}
--   - When content_is_default, it means that if there are no two parts, the content will be the first_part
function prep_inst_and_content(content, separator, options)
	local content_is_default = options and options.content_is_default or false
	local first_part, second_part = aip.text.split_first(content, separator)

	local inst, content = nil, nil
	if second_part ~= nil then
		inst = first_part
		content = second_part
	elseif content_is_default then
		content = first_part
	else
		inst = first_part
	end

	return inst, content
end

-- This loads maps the FileInfo array as a FileRecord array by loading each file
-- It also augments the FileRecord with `.comment_file_path` (.e.g., "// file: some/path/to/file.ext")
-- returns nil if refs is nil
function load_file_refs(base_dir, refs)
	local files = nil
	if refs ~= nil then
		files = {}
		for _, file_ref in ipairs(refs) do
			local file = aip.file.load(file_ref.path, { base_dir = base_dir })
			-- Augment the file with the comment file path
			file.comment_file_path = aip.code.comment_line(file.ext, "file: " .. file.path)
			table.insert(files, file)
		end
	end
	return files
end

-- Do a shallow clone, and optionally merge the to_merge table
-- original: (required) The original table to copy
-- to_merge: (optional) The optional table to merge
function shallow_copy(original, to_merge)
	local copy = {}

	-- First, copy all elements from original
	for k, v in pairs(original) do
		copy[k] = v
	end

	-- If to_merge is provided, override/add elements
	if to_merge then
		for k, v in pairs(to_merge) do
			copy[k] = v
		end
	end

	return copy
end

return {
	prep_prompt_file      = prep_prompt_file,
	should_skip           = should_skip,
	prep_inst_and_content = prep_inst_and_content,
	load_file_refs        = load_file_refs,
	shallow_copy          = shallow_copy
}
