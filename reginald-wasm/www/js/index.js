import init, {Regex} from "../../pkg/reginald_wasm.js";

init()
    .then(() => {
        let regex = null;
        let outputText = document.getElementById("output-text");

        let compileButton = document.getElementById("compile-button")
        let compileInput = document.getElementById("compile-input");
        let compileError = document.getElementById("compile-error");
        let compileGraph = document.getElementById("compile-graph");

        compileButton.addEventListener("click", () => {
            outputText.innerHTML = "";

            try {
                regex = new Regex(compileInput.value);
            } catch (err) {
                console.log(err);
                regex = null;

                compileError.innerHTML = "Error:\n" + err;
                compileGraph.innerHTML = "";
            }

            if (regex) {
                let insertSvg = function(svgCode, bindFunctions){
                    compileGraph.innerHTML = svgCode; 
                }; 
                let graph = mermaid.mermaidAPI.render('mermaid', regex.to_string(), insertSvg);

                compileError.innerHTML = "";
            }
        });

        let runButton = document.getElementById("run-button");
        let runType = document.getElementById("run-type");
        let runText = document.getElementById("run-text");

        runButton.addEventListener("click", () => {
            outputText.innerHTML = "";

            if (regex) {
                let text = runText.innerText;
                switch (runType.value) {
                    case "Match":
                        {
                            let slice = regex.is_match(text);
                            outputText.innerText = text.slice(slice.start, slice.start+slice.size);
                        }
                        break;
                    case "Matches":
                        {
                            let slices = regex.matches(text);
                            let len = slices.len();
                            for (let i = 0; i < len; i++) {
                                let slice = slices.get(i);
                                outputText.innerText += text.slice(slice.start, slice.start+slice.size) + "\n";
                            }
                        }
                        break;
                    case "Test":
                        {
                            let is = regex.test(text);
                            outputText.innerText = is ? "true" : "false";
                        }
                        break;
                }
            }
        });

        let replaceButton = document.getElementById("replace-button");
        let replaceInput = document.getElementById("replace-input");

        replaceButton.addEventListener("click", () => {
            outputText.innerHTML = "";
            if (regex) {
                let text = runText.innerText;
                let replace_text = replaceInput.value;
                let output = "";
                let slices = regex.matches(text);
                let len = slices.len();
                let text_index = 0;
                for (let i = 0; i < len; i++) {
                    let slice = slices.get(i);
                    if (text_index < slice.start) {
                        output += text.slice(text_index, slice.start);
                        text_index = slice.start;
                    }
                    output += replace_text;
                    text_index += slice.size;
                }

                if (text_index < text.length) {
                    output += text.slice(text_index, text.length);
                }

                outputText.innerText = output;
            }
        });

        mermaid.initialize({ startOnLoad: true });
        mermaid.mermaidAPI.render("mermaid", "flowchart TD\n", svgCode => compileGraph.innerHTML = svgCode)
    })