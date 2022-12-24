import { GraphQLClient } from "graphql-request";
import { getSdk } from "./api_types";

export default getSdk(new GraphQLClient('http://localhost:8080/api'))

