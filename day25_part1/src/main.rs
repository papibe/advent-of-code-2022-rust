use std::collections::{HashMap, VecDeque};
use std::fs;
// use std::io;

enum OperationType {
    SUM = 1,
    MUL = 2,
    CPY = 3,
    OUT = 4,
    JIT = 5,
    JIF = 6,
    LTH = 7,
    EQL = 8,
    ARB = 9,
    END = 99,
}

impl OperationType {
    fn from_i64(number: i64) -> OperationType {
        match number {
            1 => OperationType::SUM,
            2 => OperationType::MUL,
            3 => OperationType::CPY,
            4 => OperationType::OUT,
            5 => OperationType::JIT,
            6 => OperationType::JIF,
            7 => OperationType::LTH,
            8 => OperationType::EQL,
            9 => OperationType::ARB,
            99 => OperationType::END,
            _ => panic!("Unknown operation: {}", number),
        }
    }
}

#[derive(Debug)]
enum ParameterMode {
    PositionMode = 0,
    ImmediateMode = 1,
    RelativeMode = 2,
}

impl ParameterMode {
    fn from_i64(number: i64) -> ParameterMode {
        match number {
            0 => ParameterMode::PositionMode,
            1 => ParameterMode::ImmediateMode,
            2 => ParameterMode::RelativeMode,
            _ => panic!("Unknown parameter mode: {}", number),
        }
    }
}

struct Operation {
    operation: OperationType,
    first_parameter_mode: ParameterMode,
    second_parameter_mode: ParameterMode,
    third_parameter_mode: ParameterMode,
}

// IntcodeComputer 'class'
struct IntcodeComputer {
    program: HashMap<i64, i64>,
    pointer: i64,
    halted: bool,
    relative_base: i64,
}

impl IntcodeComputer {
    fn run(&mut self, input: &mut VecDeque<i64>) -> Vec<i64> {
        let mut output: Vec<i64> = vec![];
        loop {
            let operation: Operation = self.parse_instruction();

            match operation.operation {
                OperationType::SUM => self.sum(operation),
                OperationType::MUL => self.mul(operation),
                OperationType::CPY => {
                    if input.len() == 0 {
                        return output;
                    }
                    self.cpy(input, operation);
                }
                OperationType::OUT => self.out(operation, &mut output),
                OperationType::JIT => self.jit(operation),
                OperationType::JIF => self.jif(operation),
                OperationType::LTH => self.lth(operation),
                OperationType::EQL => self.eql(operation),
                OperationType::ARB => self.arb(operation),
                OperationType::END => break,
            }
        }
        self.halted = true;
        output
    }

    fn parse_instruction(&self) -> Operation {
        let instruction = self.program[&self.pointer];
        let operation: i64 = instruction % 100;
        let parameters: i64 = instruction / 100;

        let first_parameter_mode: i64 = parameters % 10;
        let parameters: i64 = parameters / 10;
        let second_parameter_mode: i64 = parameters % 10;
        let parameters: i64 = parameters / 10;
        let third_parameter_mode: i64 = parameters % 10;

        Operation {
            operation: OperationType::from_i64(operation),
            first_parameter_mode: ParameterMode::from_i64(first_parameter_mode),
            second_parameter_mode: ParameterMode::from_i64(second_parameter_mode),
            third_parameter_mode: ParameterMode::from_i64(third_parameter_mode),
        }
    }

    fn sum(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        let result_index: i64 = match operation.third_parameter_mode {
            ParameterMode::PositionMode => *self.program.entry(self.pointer + 3).or_insert(0),
            ParameterMode::RelativeMode => {
                self.relative_base + *self.program.entry(self.pointer + 3).or_insert(0)
            }
            _ => panic!(
                "Incorrect third parameter mode: {:?}",
                operation.third_parameter_mode
            ),
        };

        self.program.insert(result_index, parameter1 + parameter2);
        self.pointer += 4;
    }

    fn get_parameter(&mut self, parameter_mode: ParameterMode, offset: i64) -> i64 {
        match parameter_mode {
            ParameterMode::PositionMode => {
                let index: i64 = *self.program.entry(self.pointer + offset).or_insert(0);
                return *self.program.entry(index).or_insert(0);
            }
            ParameterMode::ImmediateMode => {
                return *self.program.entry(self.pointer + offset).or_insert(0)
            }
            ParameterMode::RelativeMode => {
                let index: i64 =
                    self.relative_base + *self.program.entry(self.pointer + offset).or_insert(0);
                return *self.program.entry(index).or_insert(0);
            }
        }
    }

    fn get_first_parameter(&mut self, first_parameter_mode: ParameterMode) -> i64 {
        self.get_parameter(first_parameter_mode, 1)
    }

    fn get_second_parameter(&mut self, second_parameter_mode: ParameterMode) -> i64 {
        self.get_parameter(second_parameter_mode, 2)
    }

    fn mul(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        let result_index: i64 = match operation.third_parameter_mode {
            ParameterMode::PositionMode => *self.program.entry(self.pointer + 3).or_insert(0),
            ParameterMode::RelativeMode => {
                self.relative_base + *self.program.entry(self.pointer + 3).or_insert(0)
            }
            _ => panic!(
                "Incorrect third parameter mode: {:?}",
                operation.third_parameter_mode
            ),
        };

        self.program.insert(result_index, parameter1 * parameter2);
        self.pointer += 4;
    }

    fn cpy(&mut self, inputs: &mut VecDeque<i64>, operation: Operation) {
        let input: i64 = inputs.pop_front().unwrap();
        match operation.first_parameter_mode {
            ParameterMode::PositionMode => {
                let index: i64 = *self.program.entry(self.pointer + 1).or_insert(0);
                self.program.insert(index, input);
            }
            ParameterMode::RelativeMode => {
                let index: i64 =
                    self.relative_base + *self.program.entry(self.pointer + 1).or_insert(0);
                self.program.insert(index, input);
            }
            _ => panic!(
                "Incorrect first parameter mode: {:?}",
                operation.first_parameter_mode
            ),
        }
        self.pointer += 2;
    }

    fn out(&mut self, operation: Operation, output: &mut Vec<i64>) {
        let operand: i64 = self.get_first_parameter(operation.first_parameter_mode);
        output.push(operand);

        self.pointer += 2;
    }

    fn jit(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        if parameter1 != 0 {
            self.pointer = parameter2;
        } else {
            self.pointer += 3;
        }
    }

    fn jif(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        if parameter1 == 0 {
            self.pointer = parameter2;
        } else {
            self.pointer += 3;
        }
    }

    fn lth(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        let result_index: i64 = match operation.third_parameter_mode {
            ParameterMode::PositionMode => *self.program.entry(self.pointer + 3).or_insert(0),
            ParameterMode::RelativeMode => {
                self.relative_base + *self.program.entry(self.pointer + 3).or_insert(0)
            }
            _ => panic!(
                "Incorrect third parameter mode: {:?}",
                operation.third_parameter_mode
            ),
        };

        if parameter1 < parameter2 {
            self.program.insert(result_index, 1);
        } else {
            self.program.insert(result_index, 0);
        }
        self.pointer += 4;
    }

    fn eql(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        let parameter2: i64 = self.get_second_parameter(operation.second_parameter_mode);

        let result_index: i64 = match operation.third_parameter_mode {
            ParameterMode::PositionMode => *self.program.entry(self.pointer + 3).or_insert(0),
            ParameterMode::RelativeMode => {
                self.relative_base + *self.program.entry(self.pointer + 3).or_insert(0)
            }
            _ => panic!(
                "Incorrect third parameter mode: {:?}",
                operation.third_parameter_mode
            ),
        };

        if parameter1 == parameter2 {
            self.program.insert(result_index, 1);
        } else {
            self.program.insert(result_index, 0);
        }
        self.pointer += 4;
    }

    fn arb(&mut self, operation: Operation) {
        let parameter1: i64 = self.get_first_parameter(operation.first_parameter_mode);
        self.relative_base += parameter1;

        self.pointer += 2;
    }
}

fn parse(filename: &str) -> HashMap<i64, i64> {
    // read file
    let data = fs::read_to_string(filename).expect(&format!("File not found: {filename}"));

    // convert content into a vector of integers
    let vec_data: Vec<i64> = data
        .split(",")
        .map(|x| x.trim().parse::<i64>().unwrap())
        .collect();

    let mut program: HashMap<i64, i64> = HashMap::new();

    for (index, value) in vec_data.iter().enumerate() {
        program.insert(index as i64, *value);
    }

    program
}

// Use for play interactively and obtain pre_commands
//
// fn read_command() -> String {
//     let mut buffer = String::new();
//     let stdin = io::stdin();
//     let _ = stdin.read_line(&mut buffer);
//     buffer
// }

fn solution(filename: &str) -> i32 {
    let program = parse(filename);
    let mut computer = IntcodeComputer {
        program: program.clone(),
        pointer: 0,
        halted: false,
        relative_base: 0,
    };

    let mut input: VecDeque<i64> = VecDeque::new();

    let pre_commands = [
        "north",
        "north",
        "east",
        "east",
        "take cake",
        "west",
        "west",
        "south",
        "south",
        "south",
        "west",
        "take fuel cell",
        "west",
        "take easter egg",
        "inv",
        "east",
        "east",
        "north",
        "east",
        "take ornament",
        "east",
        "take hologram",
        "east",
        "take dark matter",
        "north",
        "north",
        "east",
        "take klein bottle",
        "north",
        "take hypercube",
        "north",
        "drop ornament",
        "drop easter egg",
        "drop hypercube",
        "drop hologram",
        "drop cake",
        "drop fuel cell",
        "drop dark matter",
        "drop klein bottle",
    ];

    let _ = computer.run(&mut input);
    for command in pre_commands {
        input.clear();
        for ascii in command.chars() {
            input.push_back(ascii as i64);
        }
        input.push_back(10);

        println!("--------------> {:?}", command);
        let _ = computer.run(&mut input);
    }

    let items = [
        "ornament",
        "easter egg",
        "hypercube",
        "hologram",
        "cake",
        "fuel cell",
        "dark matter",
        "klein bottle",
    ];

    let mut taken_items: Vec<String> = vec![];
    'outer: for selection in 0..256 {
        taken_items.clear();
        for tp in 0..8 {
            if selection & (1 << tp) != 0 {
                // println!("s {}, tp {}", selection, tp);
                let mut cmd = "take ".to_string();
                cmd.push_str(items[tp].clone());
                cmd.push_str("\n");
                taken_items.push(cmd);
            }
        }
        taken_items.push("west\n".to_string());

        input.clear();
        for command in &taken_items {
            for ascii in command.chars() {
                input.push_back(ascii as i64);
            }
        }

        let output: Vec<i64> = computer.run(&mut input);

        let mut message: String = String::new();
        for c in output {
            match char::from_u32(c as u32) {
                Some(c) => message.push(c),
                None => print!(" error {}", c),
            }
        }
        if !message.contains(&"heavier") && !message.contains(&"lighter") {
            println!("{}", message);
            break 'outer;
        }
        for cmd in &mut taken_items {
            *cmd = cmd.replace("take", "drop");
        }

        input.clear();
        for command in &taken_items {
            for ascii in command.chars() {
                input.push_back(ascii as i64);
            }
            input.push_back(10);
        }

        let _ = computer.run(&mut input);
    }
    0
}

fn main() {
    println!("{:?}", solution("./input.txt")); // 1090617344
}
