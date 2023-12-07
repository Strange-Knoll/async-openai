use async_openai::{Client, types::{CreateAssistantRequestArgs, AssistantTools, AssistantToolsFunction, ChatCompletionFunctions, CreateThreadRequestArgs, CreateMessageRequestArgs, CreateRunRequestArgs, RunStatus, StepDetails, MessageContent, RunStepDetailsToolCalls, SubmitToolOutputsRunRequest, ToolsOutputs}};
use serde_json::Value;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let username = whoami::username();

    println!("Hello, {}!", username);
    println!("I'm an example bot written to explain how functions are used with the api.");
    println!("we can have a casual conversation, or you can ask me to say \"hello world.\"");
    println!("you can exit the program by typing \"exit\"");

    //create an assistant
    let create_assistant = CreateAssistantRequestArgs::default()
        .model("gpt-3.5-turbo")
        .name("example-bot")
        .description("an example bot")
        .instructions("you are a general purpose bot. you can have a casual conversation")
        .tools(vec![                //here we create a function inside a vector of tools
            AssistantTools::Function(AssistantToolsFunction{    
                r#type: "function".to_string(),
                function: ChatCompletionFunctions{
                    name: "hello_world".to_string(),
                    description: Some("prints and returns hello world".to_string()),
                    //parameters are a json string that follows this schem
                    //you can add more arguements by copying the name obj and pasting it into properties
                    //requred is a list of required arguements by name
                    parameters: serde_json::from_str(
                        r#"{
                            "type": "object", "properties": {
                                "name": {
                                    "type": "string",
                                    "description": "the name of the person to say hello to"
                                }
                            },
                            "required": ["name"]
                        }"#
                    ).unwrap(),
                }
            })
        ])
        .build()?;

    let assistant = client
        .assistants()
        .create(create_assistant)
        .await?;

    let create_thread = CreateThreadRequestArgs::default()
        .build()?;
    let thread = client
        .threads()
        .create(create_thread)
        .await?;

    //main loop
    loop {
        println!("{}> ", username);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() == "exit" {
            //delete the assistant and the thread
            client
                .assistants()
                .delete(&assistant.id)
                .await?;
            client
                .threads()
                .delete(&thread.id)
                .await?;
            break;
        }

        //create a message
        let message = CreateMessageRequestArgs::default()
            .role("user")
            .content(input.trim())
            .build()?;
        //post input to the thread
        client
            .threads()
            .messages(&thread.id)
            .create(message)
            .await?;
        
        //create run request
        let run_request = CreateRunRequestArgs::default()
            .assistant_id(&assistant.id)
            .build()?;
        //create a run
        let run = client
            .threads()
            .runs(&thread.id)
            .create(run_request)
            .await?;

        let mut running = true;
        while running {
            //wait for 1 second
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            //retrieve the run's response
            let response = client
                .threads()
                .runs(&thread.id)
                .retrieve(&run.id)
                .await?;

            match response.status {
                //if the run status is completed
                RunStatus::Completed => {
                    //retrieve last message
                    let message = client
                        .threads()
                        .messages(&thread.id)
                        .list(&[("limit","1")])
                        .await?;
                    //loop through the content
                    for content in message.data[0].content.clone() {
                        match content{
                            MessageContent::ImageFile(_image) => {
                                //print debug message if its an image file
                                println!("image was generated but is not supported in the terminal");
                            },
                            MessageContent::Text(text) => {
                                //print the text if its text
                                println!("{}", text.text.value);
                            },
                        }
                        //set running to false to exit the while loop
                        running = false;
                    }
                },
                // if the run status requires action
                RunStatus::RequiresAction => {
                    println!("run requires action");
                    //get the latest step
                    let step = client
                        .threads()
                        .runs(&thread.id)
                        .steps(&run.id)
                        .list(&[("limit","1")])
                        .await?;

                    match step.data[0].step_details.clone() {
                        //if the action required is a message creation
                        StepDetails::MessageCreation(message) => {
                            println!("message creation");
                            //retrieve the message
                            let message = client
                                .threads()
                                .messages(&thread.id)
                                .retrieve(&message.message_creation.message_id)
                                .await?;
                            //loop through the content
                            for content in message.content {
                                match content{
                                    MessageContent::ImageFile(_image) => {
                                        //print debug message if its an image file
                                        println!("image was generated but is not supported in the terminal");
                                    },
                                    MessageContent::Text(text) => {
                                        //print the text if its text
                                        println!("message creation: {}", text.text.value);
                                    },
                                }
                            }
                        },
                        //if the action required is a tool call
                        StepDetails::ToolCalls(tool_calls) => {
                            println!("tool calls");
                            //loop through the tool calls
                            for tool_call in tool_calls.tool_calls {
                                match tool_call {
                                    //we are assuming that the only tool call we will recieved is a function call
                                    //so no other branch of tool call is implemented
                                    RunStepDetailsToolCalls::Function(func) => {
                                        println!("function");
                                        //parce the args
                                        let args:Value = serde_json::from_str(&func.function.arguments).unwrap();
                                        println!("{:#?}", args);
                                        if func.function.name == "hello_world" {
                                            println!("hello world function");
                                            //print the hello world message
                                            println!("hello, {}!", args["name"]);

                                            //create a response
                                            let response = SubmitToolOutputsRunRequest{
                                                tool_outputs: vec![ToolsOutputs{
                                                    tool_call_id: Some(func.id),
                                                    output:Some("executed hello_world function".to_string()),
                                                }]
                                            };

                                            //return a response to the run
                                            client
                                                .threads()
                                                .runs(&thread.id)
                                                .submit_tool_outputs(&run.id, response)
                                                .await?;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {println!("=");}
            }
        }

    }

    Ok(())
}
