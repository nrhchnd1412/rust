trait State{
    fn next(self: Box<Self>)->Box<dyn State>;
    fn status(&self)->&str;
}

struct Draft;
struct Approved;
struct Published;

impl State for Draft{
    fn next(self:Box<Self>)-> Box<dyn State>{
        Box::new(Approved)
    }
    
    fn status(&self)->&str{
        "DRAFT"
    }
}

impl State for Approved{
    fn next(self:Box<Self>)-> Box<dyn State>{
        Box::new(Published)
    }
    
    fn status(&self)->&str{
        "APPROVED"
    }
}

impl State for Published{
    fn next(self:Box<Self>)-> Box<dyn State>{
        self
    }
    
    fn status(&self)->&str{
        "PUBLISHED"
    }
}

struct Post{
    state: Option<Box<dyn State>>,
    content: String,
}

impl Post{
    fn new()->Post{
        Post{
            state:Some(Box::new(Draft)),
            content:String::new(),
        }
    }

    fn add_content(&mut self,content:&str){
        if let Some(ref s)=self.state{
            let status=s.status();
            if status=="DRAFT"{
                self.content.push_str(content);
            }
            else{
                println!("cannot push content while in state {}",status);
            }

        }
    }

    fn status(&self)->&str{
        self.state.as_ref().map_or("No status", |s|s.status())
    }

    fn approve(&mut self){
        if let Some(s)=self.state.take(){
            self.state=Some(s.next());
        }
    }

    fn content(&self)->&str{
        if let Some(ref s)=self.state{
            if s.status()=="PUBLISHED"{
                &self.content
            }
            else{
                ""
            }
        }
        else{
            ""
        }
    }

}

fn main(){
    let mut post = Post::new();
    println!("current state of the post is {}",post.status());
    post.add_content("this is my first content");
    println!("Present content in {} status is - {}", post.status(),post.content());
    post.approve();
    println!("Present content in {} status is - {}", post.status(),post.content());
    post.approve();
    println!("Present content in {} status is - {}", post.status(),post.content());
    post.add_content("this is my second content");
}