use nom::branch::alt;
use nom::bytes::complete::tag;

pub enum ReplCommand {
    Exit,
    Help,
    GetWeight(i32),
    ListByWeight(usize),
    GetNeighbourhood(i32),
}

impl ReplCommand {
    pub fn parse(input: &str) -> Option<Self> {
        alt((
            Self::parse_exit,
            Self::parse_help,
            Self::parse_get_weight,
            Self::parse_list_by_weight,
            Self::parse_get_neighbourhood,
        ))(input)
        .ok()
        .map(|(_, cmd)| cmd)
    }

    fn parse_exit(input: &str) -> nom::IResult<&str, ReplCommand> {
        let (input, _) = tag("exit")(input)?;
        Ok((input, ReplCommand::Exit))
    }

    fn parse_help(input: &str) -> nom::IResult<&str, ReplCommand> {
        let (input, _) = tag("help")(input)?;
        Ok((input, ReplCommand::Help))
    }

    fn parse_get_weight(input: &str) -> nom::IResult<&str, ReplCommand> {
        let (input, _) = tag("get_weight")(input)?;
        let (input, _) = nom::character::complete::multispace1(input)?;
        let (input, weight) = nom::character::complete::digit1(input)?;
        Ok((input, ReplCommand::GetWeight(weight.parse().unwrap())))
    }

    fn parse_list_by_weight(input: &str) -> nom::IResult<&str, ReplCommand> {
        let (input, _) = tag("list_by_weight")(input)?;
        let (input, _) = nom::character::complete::multispace1(input)?;
        let (input, weight) = nom::character::complete::digit1(input)?;
        Ok((input, ReplCommand::ListByWeight(weight.parse().unwrap())))
    }

    fn parse_get_neighbourhood(input: &str) -> nom::IResult<&str, ReplCommand> {
        let (input, _) = tag("get_neighbourhood")(input)?;
        let (input, _) = nom::character::complete::multispace1(input)?;
        let (input, weight) = nom::character::complete::digit1(input)?;
        Ok((
            input,
            ReplCommand::GetNeighbourhood(weight.parse().unwrap()),
        ))
    }
}
