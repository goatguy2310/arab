use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "ping cai d gi?").await?;
    
    Ok(())
}

#[command]
pub async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let res = "Hello! I am Arab (rustified)! My prefix is `~`
```help: Show this message
-------------Auto Translation--------------
translate: Toggle auto translation mode for this channel (Not available yet)
list: List all available languages (Not available yet)
lang: See the current language, default is Arabic because why not (Not available yet)
langset <lang>: Set a new language for this channel (Not available yet)
---------------Dam Vinh Hung---------------
damvinhhung: Toggle Đàm mode for this channel (Not available yet)
randam: Send a random Đàm image (Not available yet)
---------------Miscellaneous---------------
cothenoi x: Co the noi x la vua cua moi nghe
please dont x: Please DO NOT announce to the server when you are going to x
replace \"text\" \"x1\" \"x2\": Change all occurences of x1 in text to x2

miketype x: Show a picture of Mike typing/looking at something (Not available yet)
getthismana x: Show a picture of Black Panther saying \"Get this man a x\" (Not available yet)
-------------------Voice-------------------
tts x: Join and play text-to-speech message (Not available yet)
join: Join voice channel
leave: leave voice channel

play x: Play music from youtube link or search queries on youtube
queue: Show the current queue
current: Show the current song
skip: Skip the current song
remove x: Remove the xth song from the queue
repeat: Repeat the current queue
pause: Pause the current song
resume: Resume the current song
```";
    msg.channel_id.say(&ctx.http, res).await?;

    Ok(())
}

#[command]
pub async fn cothenoi(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.rest();
    let result = "Có thể nói among là vua của các nghề

Ngành among Việt Nam hiện nay ở đầu của sự phát triển. Có thể nói among là vua của các nghề. Vừa có tiền, có quyền. Vừa kiếm được nhiều $ lại được xã hội trọng vọng.
    
Thằng em mình học bách khoa cơ khí, bỏ ngang sang among, sinh năm 96. Tự mày mò học among rồi đi làm remote cho công ty Mỹ 2 năm nay. Mỗi tối online 3-4 giờ là xong việc. Lương tháng 3k6. Nhưng thu nhập chính vẫn là từ nhận các project bên ngoài làm thêm. Tuần làm 2,3 cái nhẹ nhàng 9,10k tiền tươi thóc thật không phải đóng thuế. Làm gần được 3 năm mà nhà xe nó đã mua đủ cả. Nghĩ mà thèm.
    
Gái gú thì cứ nghe nó bảo làm among thì chảy nước. Có bé kia dân du học sinh Úc, về được cô chị giới thiệu làm ngân hàng VCB. Thế nào thằng ấy đi mở thẻ tín dụng gặp phải thế là hốt được cả chị lẫn em. 3 đứa nó sống chung một căn hộ cao cấp. Nhà con bé kia biết chuyện ban đầu phản đối sau biết thằng đấy học among thì đổi thái độ, cách ba bữa hỏi thăm, năm bữa tặng quà giục cưới kẻo lỡ kèo. Đáng lẽ tháng này là đám cưới tụi nó nhưng dính covid nên dời lại cuối năm rồi.
".replace("among", one);
 
    msg.channel_id.say(&ctx.http, result).await?;

    Ok(())
}

#[command]
pub async fn pleasedont(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.rest();
    let result = "Please DO NOT announce to the server when you are going to among. This has been a reoccurring issue, and l'm not sure why some people have such under developed social skills that they think that a server full of mostly male strangers would need to know that. No one is going to be impressed and give you a high five (especially considering where that hand has been). I don't want to add this to the rules, since it would be embarrassing for new users to see that we have a problem with this, but it is going to be enforced as a rule from now on.

If it occurs, you will be warned, then additional occurrences will be dealt with at the discretion of modstaff. Thanks.
".replace("among", one);
 
    msg.channel_id.say(&ctx.http, result).await?;

    Ok(())
}

#[command]
pub async fn replace(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let txt = args.single::<String>().unwrap();
    let a = args.single::<String>().unwrap();
    let b = args.single::<String>().unwrap();
    let result = txt.replace(&a, &b);
 
    msg.channel_id.say(&ctx.http, result).await?;

    Ok(())
}         