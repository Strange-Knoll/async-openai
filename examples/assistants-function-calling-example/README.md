# Assistants Function Calling Example
This example is written to show how to create functions, attach them to an assistant, recieve function calls, and return the results of the function call to the assistant.

This example creates a simple assistant that is a generic chat bot with a single function "hello_world". To call this function ask it something like ```call your hello world function```.

The process is fairly straight forward.
1. create an assistant and create the functions for that assisstant
2. await user input and a response from the assistant
3. if the assistant calls a function we parce which function was called
4. call the function in our own code
5. return the results of our function call to the assistant.

We create an assistant with a function in the following way:
```
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
```
Notice that ```.tools()``` takes a vector of AssistantTools. These tools can be multiple different function calls, retrevial, or code. We are only interested in creating functions in this example.

Once we have determined we are calling a function, and which function is being called we can run our function and return its results to the assistant as follows:
```
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
```
The entire process is broken down in detail in the examples source code. 

To run this example, clone it from the repository, 
set your api key as an environment variable ```export OPENAI_API_KEY="sk-..."```,
and run the example with ```cargo run```