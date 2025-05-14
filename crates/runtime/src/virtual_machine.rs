//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner/VirtualMachine.cs>
//!
//! ## Implementation Notes
//! The `Operand` extensions and the `Operator` enum were moved into upstream crates to make them not depend on the runtime.

pub(crate) use self::{execution_state::*, state::*};
use crate::prelude::*;
use crate::Result;
use core::fmt::Debug;
use log::*;
use yarnspinner_core::prelude::instruction::{AddOptionInstruction, CallFunctionInstruction, InstructionType, JumpIfFalseInstruction, JumpToInstruction, PushBoolInstruction, PushFloatInstruction, PushStringInstruction, PushVariableInstruction, RunCommandInstruction, RunLineInstruction, RunNodeInstruction, StoreVariableInstruction};

mod execution_state;
mod state;

#[derive(Debug, Clone)]
pub(crate) struct VirtualMachine {
    pub(crate) library: Library,
    pub(crate) program: Option<Program>,
    pub(crate) variable_storage: Box<dyn VariableStorage>,
    current_node_name: Option<String>,
    state: State,
    execution_state: ExecutionState,
    current_node: Option<Node>,
    batched_events: Vec<DialogueEvent>,
}

impl VirtualMachine {
    pub(crate) fn new(
        library: Library,
        variable_storage: Box<dyn VariableStorage>,
    ) -> Self {
        Self {
            library,
            variable_storage,
            program: Default::default(),
            current_node_name: Default::default(),
            state: Default::default(),
            execution_state: Default::default(),
            current_node: Default::default(),
            batched_events: Default::default(),
        }
    }

    pub(crate) fn variable_storage(&self) -> &dyn VariableStorage {
        self.variable_storage.as_ref()
    }

    pub(crate) fn variable_storage_mut(&mut self) -> &mut dyn VariableStorage {
        self.variable_storage.as_mut()
    }

    pub(crate) fn reset_state(&mut self) {
        self.state = State::default();
        self.current_node_name = None;
    }

    pub(crate) fn set_execution_state(&mut self, execution_state: ExecutionState) -> &mut Self {
        self.execution_state = execution_state;
        if execution_state == ExecutionState::Stopped {
            self.reset_state()
        }
        self
    }

    /// # Implementation Notes
    /// The original does not reset the state upon calling this. I suspect that's a bug.
    pub(crate) fn stop(&mut self) -> Vec<DialogueEvent> {
        self.set_execution_state(ExecutionState::Stopped);
        self.batched_events.push(DialogueEvent::DialogueComplete);
        core::mem::take(&mut self.batched_events)
    }

    pub(crate) fn set_node(&mut self, node_name: impl Into<String>) -> Result<()> {
        let node_name = node_name.into();
        debug!("Loading node \"{node_name}\"");
        let current_node = self.get_node_from_name(&node_name)?;
        self.current_node = Some(current_node.clone());

        self.reset_state();

        self.current_node_name = Some(node_name.clone());

        self.batched_events
            .push(DialogueEvent::NodeStart(node_name));

        Ok(())
    }

    fn get_node_from_name(&self, node_name: &str) -> Result<&Node> {
        let program = self
            .program
            .as_ref()
            .ok_or_else(|| DialogueError::NoProgramLoaded)?;

        program
            .nodes
            .get(node_name)
            .ok_or_else(|| DialogueError::InvalidNode {
                node_name: node_name.to_owned(),
            })
    }

    /// Resumes execution.
    pub(crate) fn continue_(
        &mut self,
        mut instruction_fn: impl FnMut(&mut Self, &Instruction) -> crate::Result<()>,
    ) -> crate::Result<Vec<DialogueEvent>> {
        self.assert_can_continue()?;
        self.set_execution_state(ExecutionState::Running);

        while self.execution_state == ExecutionState::Running {
            let current_node = self.current_node.clone().unwrap();
            let current_instruction = &current_node.instructions[self.state.program_counter as usize];
            instruction_fn(self, current_instruction)?;
            // ## Implementation note
            // The original increments the program counter here, but that leads to intentional underflow on [`OpCode::RunNode`],
            // so we do the incrementation in [`VirtualMachine::run_instruction`] instead.

            if self.state.program_counter < current_node.instructions.len() {
                continue;
            }

            self.batched_events
                .push(DialogueEvent::NodeComplete(current_node.name.clone()));
            self.set_execution_state(ExecutionState::Stopped);
            self.batched_events.push(DialogueEvent::DialogueComplete);
            debug!("Run complete.");
        }
        Ok(core::mem::take(&mut self.batched_events))
    }

    /// Runs a series of tests to see if the [`VirtualMachine`] is in a state where [`VirtualMachine::r#continue`] can be called. Panics if it can't.
    pub(crate) fn assert_can_continue(&self) -> crate::Result<()> {
        if self.current_node.is_none() || self.current_node_name.is_none() {
            Err(DialogueError::NoNodeSelectedOnContinue)
        } else if self.execution_state == ExecutionState::WaitingOnOptionSelection {
            Err(DialogueError::ContinueOnOptionSelectionError)
        } else {
            // ## Implementation note:
            // The other checks the original did are not needed because our relevant handlers cannot be `None` per our API.
            Ok(())
        }
    }

    pub(crate) fn unload_programs(&mut self) {
        self.program = None
    }

    pub(crate) fn set_selected_option(&mut self, selected_option_id: OptionId) -> Result<()> {
        if self.execution_state != ExecutionState::WaitingOnOptionSelection {
            return Err(DialogueError::UnexpectedOptionSelectionError);
        }
        if selected_option_id.0 >= self.state.current_options.len() {
            return Err(DialogueError::InvalidOptionIdError {
                selected_option_id,
                max_id: self.state.current_options.len().saturating_sub(1),
            });
        }

        // We now know what number option was selected; push the
        // corresponding node name to the stack.
        let destination_node = self.state.current_options[selected_option_id.0]
            .destination_node
            .clone();
        self.state.push(destination_node);

        // We no longer need the accumulated list of options; clear it
        // so that it's ready for the next one
        self.state.current_options.clear();

        // We're no longer in the WaitingForOptions state; we are now waiting for our game to let us continue
        self.set_execution_state(ExecutionState::WaitingForContinue);
        Ok(())
    }

    pub(crate) fn is_active(&self) -> bool {
        self.execution_state != ExecutionState::Stopped
    }

    pub(crate) fn is_waiting_for_option_selection(&self) -> bool {
        self.execution_state == ExecutionState::WaitingOnOptionSelection
    }

    pub(crate) fn current_node(&self) -> Option<String> {
        self.current_node_name.clone()
    }

    /// ## Implementation note
    ///
    /// Increments the program counter here instead of in `continue_` for cleaner code
    pub(crate) fn run_instruction(
        &mut self,
        instruction: &Instruction,
        mut function_call_fn: impl FnMut(&dyn UntypedYarnFn, Vec<YarnValue>) -> YarnValue,
    ) -> crate::Result<()> {
        let Some(instruction_type) = &instruction.instruction_type else {
            panic!("Instruction type is None");
        };

        match instruction_type {
            InstructionType::JumpTo(JumpToInstruction { destination }) => {
                // Jumps to a named label
                self.state.program_counter = *destination as usize;
            }
            InstructionType::PeekAndJump(_) => {
                let jump_destination: usize = self.state.peek();
                self.state.program_counter = jump_destination;
            }
            InstructionType::RunLine(RunLineInstruction { line_id, substitution_count }) => {
                // Looks up a string from the string table and passes it to the client as a line

                let string_id: LineId = line_id.into();

                // The second operand, if provided (compilers prior
                // to v1.1 don't include it), indicates the number
                // of expressions in the line. We need to pop these
                // values off the stack and deliver them to the
                // line handler.
                for _ in 0..*substitution_count {
                    self.state.pop_value();
                }

                self.batched_events.push(DialogueEvent::Line(Line { id: string_id }));

                // Implementation note:
                // In the original, this is only done if `execution_state` is still `DeliveringContent`,
                // because the line handler is allowed to call `continue_`. However, we disallow that because of
                // how this violates borrow checking. So, we'll always wait at this point instead until the user
                // called `continue_` themselves outside of the line handler.
                self.set_execution_state(ExecutionState::WaitingForContinue);
                self.state.program_counter += 1;
            }
            InstructionType::RunCommand(RunCommandInstruction { command_text, substitution_count }) => {
                // Passes a string to the client as a custom command
                let command_text = (0..*substitution_count)
                    .map(|_| self.state.pop::<String>())
                    .enumerate()
                    .fold(command_text.to_owned(), |command_text, (i, substitution)| {
                        command_text.replace(&format!("{{{i}}}"), &substitution)
                    });
                let command = Command::parse(command_text);

                self.batched_events.push(DialogueEvent::Command(command));

                // Implementation note:
                // In the original, this is only done if `execution_state` is still `DeliveringContent`,
                // because the line handler is allowed to call `continue_`. However, we disallow that because of
                // how this violates borrow checking. So, we'll always wait at this point instead until the user
                // called `continue_` themselves outside of the line handler.
                self.set_execution_state(ExecutionState::WaitingForContinue);
                self.state.program_counter += 1;
            }
            InstructionType::AddOption(AddOptionInstruction { line_id, destination, has_condition, .. }) => {
                // TODO: Do something with substitution_count

                // Add an option to the current state
                let line = Line { id: line_id.into() };

                // Indicates whether the VM believes that the
                // option should be shown to the user, based on any
                // conditions that were attached to the option.
                let line_condition_passed = if *has_condition {
                    // The fourth operand is a bool that indicates
                    // whether this option had a condition or not.
                    // If it does, then a bool value will exist on
                    // the stack indicating whether the condition
                    // passed or not. We pass that information to
                    // the game.
                    self.state.pop()
                } else {
                    true
                };

                let index = self.state.current_options.len();
                // ## Implementation note:
                // The original calculates the ID in the `ShowOptions` opcode,
                // but this way is cleaner because it allows us to store a `DialogueOption` instead of a bunch of values in a big tuple.
                self.state.current_options.push(DialogueOption {
                    line,
                    id: OptionId(index),
                    destination_node: *destination,
                    is_available: line_condition_passed,
                });
                self.state.program_counter += 1;
            }
            InstructionType::ShowOptions(_) => {
                // If we have no options to show, immediately stop.
                if self.state.current_options.is_empty() {
                    self.batched_events.push(DialogueEvent::DialogueComplete);
                    self.set_execution_state(ExecutionState::Stopped);
                    self.state.program_counter += 1;
                    return Ok(());
                }

                // We can't continue until our client tell us which option to pick
                self.set_execution_state(ExecutionState::WaitingOnOptionSelection);

                // Pass the options set to the client, as well as a
                // delegate for them to call when the user has made
                // a selection
                let current_options = self.state.current_options.clone();
                self.batched_events
                    .push(DialogueEvent::Options(current_options));

                // Implementation note:
                // Not checking the execution state now since we have no line handler to call `continue_` from.
                self.state.program_counter += 1;
            }
            InstructionType::PushString(PushStringInstruction { value }) => {
                // Pushes a string value onto the stack.
                self.state.push(value.to_owned());
                self.state.program_counter += 1;
            }
            InstructionType::PushFloat(PushFloatInstruction { value }) => {
                // Pushes a floating point onto the stack.
                self.state.push(*value);
                self.state.program_counter += 1;
            }
            InstructionType::PushBool(PushBoolInstruction { value }) => {
                // Pushes a boolean value onto the stack.
                self.state.push(*value);
                self.state.program_counter += 1;
            }

            InstructionType::JumpIfFalse(JumpIfFalseInstruction { destination }) => {
                // Jumps to a named label if the value on the top of the stack evaluates to the boolean value 'false'.
                let is_top_value_true: bool = self.state.peek();
                if is_top_value_true {
                    self.state.program_counter += 1;
                } else {
                    self.state.program_counter = *destination as usize;
                }
            }
            InstructionType::Pop(_) => {
                // Pops a value from the stack.
                self.state.pop_value();
                self.state.program_counter += 1;
            }
            InstructionType::CallFunc(CallFunctionInstruction { function_name }) => {
                let actual_parameter_count: usize = self.state.pop();
                // Get the parameters, which were pushed in reverse
                let parameters = {
                    let mut parameters: Vec<_> = (0..actual_parameter_count)
                        .map(|_| self.state.pop_value().raw_value)
                        .collect();
                    parameters.reverse();
                    parameters
                };

                // Call a function, whose parameters are expected to be on the stack. Pushes the function's return value, if it returns one.
                let function =
                    self.library
                        .get(&function_name)
                        .ok_or(DialogueError::FunctionNotFound {
                            function_name: function_name.to_string(),
                            library: self.library.clone(),
                        })?;

                // Expect the compiler to have placed the number of parameters
                // actually passed at the top of the stack.
                let expected_parameter_count = function.parameter_types().len();

                assert_eq!(
                    expected_parameter_count, actual_parameter_count,
                    "Function {function_name} expected {expected_parameter_count} parameters, but received {actual_parameter_count}",
                );

                // Invoke the function
                let return_value = function_call_fn(function, parameters);
                let return_type = function
                    .return_type()
                    .try_into()
                    .unwrap_or_else(|e| panic!("Failed to get Yarn type for return type id of function {function_name}: {e:?}"));
                let typed_return_value = InternalValue {
                    raw_value: return_value,
                    r#type: return_type,
                };
                // ## Implementation note:
                // The original code first checks whether the return type is `void`. This is vestigial from the v1 compiler.
                // In current Yarn, every function MUST return a valid typed value, so we skip that check.
                self.state.push(typed_return_value);
                self.state.program_counter += 1;
            }
            InstructionType::PushVariable(PushVariableInstruction { variable_name }) => {
                // Get the contents of a variable, push that onto the stack.
                let loaded_value = self
                    .variable_storage
                    .get(variable_name)
                    .or_else(|e| {
                        if let VariableStorageError::VariableNotFound { .. } = e {
                            // We don't have a value for this. The initial
                            // value may be found in the program. (If it's
                            // not, then the variable's value is undefined,
                            // which isn't allowed.)
                            let initial_value = self
                                .program
                                .as_ref()
                                .unwrap()
                                .initial_values
                                .get(variable_name)
                                .unwrap_or_else(|| panic!("The loaded program does not contain an initial value for the variable {variable_name}"))
                                .clone();

                            // Store the initial value in the variable_storage
                            self.variable_storage.set(variable_name.clone(), initial_value.clone().into())?;

                            Ok(initial_value.into())
                        } else {
                            Err(e)
                        }
                    })?;
                self.state.push(loaded_value);
                self.state.program_counter += 1;
            }
            InstructionType::StoreVariable(StoreVariableInstruction { variable_name }) => {
                // Store the top value on the stack in a variable.
                let top_value = self.state.peek_value().clone();
                self.variable_storage.set(variable_name.to_owned(), top_value.into())?;
                self.state.program_counter += 1;
            }
            InstructionType::Stop(_) => {
                // Immediately stop execution, and report that fact.
                let current_node_name = self.current_node_name.clone().unwrap();
                self.batched_events
                    .push(DialogueEvent::NodeComplete(current_node_name));
                self.batched_events.push(DialogueEvent::DialogueComplete);
                self.set_execution_state(ExecutionState::Stopped);

                self.state.program_counter += 1;
            }
            InstructionType::RunNode(RunNodeInstruction { node_name }) => {
                // Run a node

                self.batched_events
                    .push(DialogueEvent::NodeComplete(node_name.to_owned()));
                self.set_node(node_name)?;

                // No need to increment the program counter, since otherwise we'd skip the first instruction
                // TODO: Reset program counter?
            }
            InstructionType::PeekAndRunNode(_) => {
                let node_name: String = self.state.pop();
                self.set_node(node_name)?;
            }
            InstructionType::DetourToNode(_) => {
                unimplemented!("DetourToNode is not implemented yet")
            }
            InstructionType::PeekAndDetourToNode(_) => {
                unimplemented!("PeekAndDetourToNode is not implemented yet")
            }
            InstructionType::Return(_) => {
                unimplemented!("Return is not implemented yet")
            }
            InstructionType::AddSaliencyCandidate(_) => {
                unimplemented!("AddSaliencyCandidate is not implemented yet")
            }
            InstructionType::AddSaliencyCandidateFromNode(_) => {
                unimplemented!("AddSaliencyCandidateFromNode is not implemented yet")
            }
            InstructionType::SelectSaliencyCandidate(_) => {
                unimplemented!("SelectSaliencyCandidate is not implemented yet")
            }
        }
        Ok(())
    }
}