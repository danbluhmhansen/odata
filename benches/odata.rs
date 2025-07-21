use odata::OData;

fn main() {
    divan::main();
}

const ODATAS: &[&str] = &[
    concat!(
        "$count&",
        "$expand=rel($count&select=sub)",
        "$filter=name eq 'milk'&",
        "$orderby=one,two asc,three desc&",
        "$search=In sapiente atque eum molestias eos.&",
        "$select=one,two,three&",
        "$skip=20&",
        "$top=10&",
    ),
    concat!(
        "$filter=OrderDate ge 2023-01-01 and OrderDate le 2023-12-31 and ",
        "ShipCountry eq 'USA' and ",
        "(ShipCity eq 'Seattle' or ShipCity eq 'New York') and ",
        "(CustomerID eq 'ALFKI' or CustomerID eq 'ANATR')&",
        "$orderby=OrderDate desc, ShipCity asc&",
        "$select=OrderID, OrderDate, ShipName, ShipCity, ShipCountry, CustomerID&",
        "$expand=Order_Details($select=ProductID, UnitPrice, Quantity, Discount)&",
        "$count=true&",
        "$top=10&",
        "$skip=5&",
    ),
];

#[divan::bench(args = ODATAS)]
fn odata<'src>(s: &'src str) -> Result<OData<'src>, String> {
    OData::try_from(s)
}
