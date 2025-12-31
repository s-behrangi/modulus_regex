//import init, { mod_regex } from './pkg';

import initSync, { mod_regex } from './pkg/lib.js';

async function run() {
    await initSync();

    const divisor = document.getElementById("divisor");
    const remainder = document.getElementById("remainder");
    const base = document.getElementById("base");
    const submit = document.getElementById("submit-btn");
    const output = document.getElementById("output");
    const copy = document.getElementById("copy-btn");
    const sizeWarning = document.getElementById("size-warning");

    let input_numeric = true;


    [divisor, remainder, base].forEach((field, index) => {
        field.addEventListener("input", (event) => {
            console.log(`Field ${index + 1} changed to:`, event.target.value);
            sizeWarning.innerHTML = "";
            if ([divisor.value, remainder.value, base.value].map(val => validateInput(Number(val))).some(val => !val)) {
                input_numeric = false;
            } else {
                input_numeric = true;
                if (Number(divisor.value) > 30) {
                    sizeWarning.innerHTML = "Warning: likely to hang";
                    sizeWarning.style.color = "red";
                }  else if (Number(divisor.value) > 15) {
                    sizeWarning.innerHTML = "Caution: this d may hang for b >> 2";
                    sizeWarning.style.color = "orange";
                }
            }
        });
    });

    submit.addEventListener("click", () => {
        console.log(submit);
        
        if (!input_numeric) {
            output.value = "Input restricted to non-negative integers...";
        } else{
            const d =  Number(divisor.value);
            const r =  Number(remainder.value);
            const b =  Number(base.value);

            if (d == 0) {
                output.value = "Cannot divide by 0 sorry :]";
            } else if (r >= d) {
                output.value = "Remainder must be less than divisor, since n % d yields equivalence class r in [0..d)";
            } else if (b == 0){
                output.value = "Base 0?";
            } else if (b > 16) {
                output.value = "Bases only permitted up to 16 to allow hexadecimal encoding :/";
            } else {
                console.log("Function yields: ", mod_regex(d, b, r));
                output.value = mod_regex(d, b, r);
            }
        }
    });

    copy.addEventListener("click", () => {
        output.select();
        output.setSelectionRange(0, 99999); // For mobile devices
        navigator.clipboard.writeText(output.value);
    });
}

function validateInput(n) {
    if (Number.isInteger(n) && n >= 0) {
        return true;
    }
    return false;
}

run();