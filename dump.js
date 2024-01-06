function dump() {
  let tb = document.querySelectorAll('tbody')[0];
  let trs = [...tb.querySelectorAll('tr')];
  let result = trs
    .map((tr) => {
      let tds = tr.querySelectorAll('td');
      let name = get_content(tds[0]);
      let code = get_content(tds[1]);
      let type = get_content(tds[2]);
      let names = name.split(' ');
      let [arg, ret] = type.split('->');
      let item = {
        name: names[0].replace('.', '_'),
        caml: name_to_caml(names[0]),
        immediate: names.slice(1),
        code: code.split(' '),
        arg: [...arg.split(' ')].map(remove_bracket).filter(Boolean),
        ret: [remove_bracket(ret)].filter(Boolean),
        validation: tds[3].querySelector('a')?.href,
        execution: tds[4].querySelector('a')?.href,
      };

      return item;
    })
    .filter((item) => item.name !== '(reserved)');

  return result;
}

function get_content(td) {
  return td.textContent
    .replaceAll('∗', '*')
    .replaceAll('  ', ' ')
    .replaceAll(' ', ' ')
    .replace('→', '->');
}

function remove_bracket(text = '') {
  return text.replaceAll('[', '').replaceAll(']', '');
}

function name_to_caml(name) {
  let result = name[0].toUpperCase();

  for (let i = 1; i < name.length; i++) {
    const char = name[i];
    if (char === '.' || char === '_') {
      result += name[++i].toUpperCase();
    } else {
      result += char;
    }
  }

  return result;
}

function replace_im(text) {
  switch (text) {
    case 'bt':
      return 'BlockArg';
    case 'laneidx16':
      return 'LaneIdx16';
    case '0x01':
      return 'u8';
    case 'x':
    case 'y':
      return 'u32';
    case 'l*, l':
      return 'BrTableArg';
    case 'l':
      return 'LabelIdx';
    case 'memarg':
      return 'MemArg';
    case 'laneidx':
      return 'LaneIdx';
    case 't':
      return 'u64';
    default:
      return text;
  }
}

function dump_enum() {
  let enums = instructions.map((instr) => {
    let name = instr.caml;
    let code = instr.code
      .slice(0, 2)
      .map((item) => item.replace('0x', '').toLowerCase())
      .join('');

    if (instr.immediate.length) {
      name += `(${instr.immediate.map(replace_im).join(', ')}) `;
    } else if (instr.code.length === 3) {
      name += `(${replace_im(instr.code[2])}) `;
    }

    return (
      name + ' = 0x' + code + ', // ' + instr.name + ' ' + instr.code.join(' ')
    );
  });

  return enums.join('\n');
}

function pop_type(type) {
  switch (type) {
    case 'value':
      return '';
    default:
      return '_' + type;
  }
}

function instr_fn() {
  let code = instructions.map((instr) => {
    let code = `///${instr.execution}\npub fn ${instr.name}(&mut self`;

    if (instr.immediate.length) {
      code += ', ';
    }

    let immediate = instr.immediate
      .map((item) => `${item}: ${replace_im(item)}`)
      .join(', ');

    code += immediate + '){';

    let pop = instr.arg.map(
      (arg, i) => `\n    let v${i + 1} = self.pop${pop_type(arg)}();`
    );

    code += pop.join('\n');

    if (instr.ret[0]) {
      code += `\n    // self.push${pop_type(instr.ret[0])}();`;
    }

    code += '\n}\n';

    return code;
  });

  return code;
}

let instructions = dump();
let code = instructions.map((instr) => {
  let code = `Instruction::${instr.caml}`;
  let immediate = instr.immediate.join(', ');

  if (instr.immediate.length) {
    code += `(${immediate})`;
  }

  code += ` => self.${instr.name}`;

  if (instr.immediate.length) {
    code += `(${immediate}),`;
  } else {
    code += '(),';
  }

  return code;
});

console.log(code.join('\n'));
