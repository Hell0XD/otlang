import React, { useEffect, useState } from 'react';
import Layout from '@theme/Layout';
import Editor from 'react-simple-code-editor';

import init, { run, compile } from "../wasm/web";

export default function EditorPage(): JSX.Element {
    let [code, setCode] = useState("");
    let [console, setConsole] = useState("");

    useEffect(() => {
        if (window != undefined) {
            (window as any).my_console = function(s: string) {
                setConsole((v) => v + s);
            };
            init();
        }
    }, [null]);

    function CompileAndRun() {
        try {
            let bytes = compile(code);
            run(bytes);
        } catch (error) {
            (window as any).my_console("Failed to compile\n");
        }
    }

    return (
        <Layout
            title={`Editor`}
            description="Editor">
            <div style={{borderStyle: "solid"}}>
                <Editor
                    value={code}
                    onValueChange={setCode}
                    highlight={highlight}
                    padding={10}
                    style={{
                        minHeight: "200px",
                        fontFamily: '"Fira code", monospace',
                        fontSize: 14,
                    }}
                />
            </div>
            <button onClick={CompileAndRun}>Run</button>
            <button onClick={() => setConsole("")}>Clear</button>
            <pre style={{overflow: "scroll", height: "100%"}}>
                {console}
            </pre>
        </Layout>
    );
}

function chars_iter(s: string) {
    let chars = s.split('');
    let i = 0;
    return () => chars[i++]
}

function highlight(code: string): string {
    const chars = chars_iter(code);

    let char = '';
    let new_code = "";

    while((char = chars()) != undefined) {
        switch (char) {
            case "(":
            case ")":
                new_code += `<b>${char}</b>`;
                break;
            default:
                new_code += char;
                break;
        }
    }

    return new_code;
}